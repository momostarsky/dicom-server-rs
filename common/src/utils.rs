use std::option::Option;

use crate::dicom_object_meta::{
    DicomImageMeta, DicomStateMeta, DicomStoreMeta, make_image_info, make_state_info,
};
use crate::message_sender::MessagePublisher;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::Whatever;
use dicom_object::ReadError;
use dicom_object::file::CharacterSetOverride;
use slog::LevelFilter;
use slog::{Drain, Logger, error, info, o};
use std::collections::HashSet;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::OnceLock;

pub async fn get_dicom_files_in_dir(p0: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = std::path::Path::new(p0);

    if path.is_file() {
        // 如果是单个文件，直接检查是否为DICOM文件
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("dcm") {
                return Ok(vec![p0.to_string()]);
            }
        }
        return Ok(vec![]);
    }

    // 如果是目录，则递归查找
    let mut dicom_files = Vec::new();
    collect_dicom_files(p0, &mut dicom_files)?;
    Ok(dicom_files)
}
pub fn collect_dicom_file(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归遍历子目录
                collect_dicom_file(&path, files);
            } else if path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("dcm"))
            {
                // 添加.dcm文件到列表
                files.push(path);
            }
        }
    }
}
// 辅助函数：递归收集DICOM文件
pub fn collect_dicom_files(
    dir_path: &str,
    dicom_files: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取目录项
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // 如果是目录，递归处理
            collect_dicom_files(&path.to_string_lossy(), dicom_files)?;
        } else if path.is_file() {
            // 如果是文件，检查扩展名
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("dcm") {
                    // 收集DICOM文件路径
                    dicom_files.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    Ok(())
}
// 全局logger静态变量
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

// 设置全局logger
pub fn set_global_logger(logger: Logger) {
    let _ = GLOBAL_LOGGER.set(logger);
}

// 获取全局logger
pub fn get_logger() -> &'static Logger {
    GLOBAL_LOGGER.get().expect("Logger not initialized")
}
// 设置日志记录，日志文件按大小滚动，保留最近7个文件
// 同时设置全局logger
pub fn setup_logging(policy_name: &str) -> Logger {
    // 创建控制台logger
    let stdout_decorator = slog_term::TermDecorator::new().build();
    let stdout_drain = slog_term::FullFormat::new(stdout_decorator).build().fuse();
    let stdout_drain = slog_async::Async::new(stdout_drain).build().fuse();

    // 创建文件logger
    fs::create_dir_all("./logs").unwrap_or(());
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("./logs/{}.log", policy_name))
        .unwrap();

    let file_decorator = slog_term::PlainDecorator::new(file);
    let file_drain = slog_term::FullFormat::new(file_decorator).build().fuse();
    let file_drain = slog_async::Async::new(file_drain).build().fuse();

    // 组合drains
    let drain = slog::Duplicate::new(stdout_drain, file_drain).map(slog::Fuse);
    let drain = LevelFilter::new(drain, slog::Level::Info).map(slog::Fuse);

    let clogger: Logger = Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION"))).into();

    set_global_logger(clogger.clone());

    clogger
}
// ... existing code ...

pub fn get_unique_tenant_ids(message: &[DicomStoreMeta]) -> Vec<String> {
    let tenant_ids: HashSet<String> = message
        .iter()
        .map(|m| String::from(m.tenant_id.as_str()))
        .collect();
    tenant_ids.into_iter().collect()
}
pub fn deduplicate_state_metas(state_metas: Vec<DicomStateMeta>) -> Vec<DicomStateMeta> {
    use std::collections::HashMap;

    let mut unique_map: HashMap<(String, String, String, String), DicomStateMeta> = HashMap::new();

    for state_meta in state_metas {
        let key = state_meta.unique_key();
        // 如果键已存在，则新值会覆盖旧值
        unique_map.insert(key, state_meta);
    }

    unique_map.into_values().collect()
}
pub async fn group_dicom_state(
    messages: &[DicomStoreMeta],
) -> Result<(Vec<DicomStateMeta>, Vec<DicomImageMeta>), ReadError> {
    let logger = get_logger();
    let logger = logger.new(o!("thread" => "group_dicom_state"));
    info!(
        logger,
        "group_dicom_state batch of {} messages",
        messages.len()
    );

    let mut state_metas: Vec<DicomStateMeta> = Vec::new();
    let mut image_entities: Vec<DicomImageMeta> = Vec::new();

    for message in messages {
        let study_uid = Option::from(message.study_uid.as_str());
        let space_size = Option::from(message.file_size);
        match dicom_object::OpenFileOptions::new()
            .charset_override(CharacterSetOverride::AnyVr)
            .read_until(tags::PIXEL_DATA)
            .open_file(String::from(message.file_path.as_str()))
        {
            Ok(dicom_obj) => {
                let state_meta =
                    match make_state_info(message.tenant_id.as_str(), &dicom_obj, study_uid) {
                        Ok(state_meta) => state_meta,
                        Err(err) => {
                            error!(
                                logger,
                                "Failed to extract state meta from file: {} , message: {:?}",
                                message.file_path.as_str(),
                                err
                            );
                            continue;
                        }
                    };
                let image_entity =
                    match make_image_info(message.tenant_id.as_str(), &dicom_obj, space_size) {
                        Ok(image_entity) => image_entity,
                        Err(err) => {
                            error!(
                                logger,
                                "Failed to extract image entity from file: {} , message: {:?}",
                                message.file_path.as_str(),
                                err
                            );
                            continue;
                        }
                    };

                state_metas.push(state_meta);

                image_entities.push(image_entity);
            }
            Err(err) => {
                error!(
                    logger,
                    "Failed to open DICOM file: {} , file_path: {}",
                    err,
                    message.file_path.as_str()
                );
            }
        }
    }
    state_metas = deduplicate_state_metas(state_metas);
    Ok((state_metas, image_entities))
}

// 发送消息到指定队列
pub async fn publish_messages(
    message_producer: &dyn MessagePublisher,
    dicom_message_lists: &[DicomStoreMeta],
) -> Result<(), Whatever> {
    if dicom_message_lists.is_empty() {
        return Ok(());
    }
    let logger = get_logger();
    let logger = logger.new(o!("thread" => "publish_messages"));
    match message_producer
        .send_batch_messages(&dicom_message_lists)
        .await
    {
        Ok(_) => {
            info!(logger, "Successfully publish_messages");
        }
        Err(e) => {
            error!(logger, "Failed to publish_messages: {}", e);
        }
    }
    Ok(())
}
pub async fn publish_state_messages(
    message_producer: &dyn MessagePublisher,
    state_metaes: &[DicomStateMeta],
) -> Result<(), Whatever> {
    if state_metaes.is_empty() {
        return Ok(());
    }
    let logger = get_logger();
    let logger = logger.new(o!("thread" => "publish_state_messages"));
    match message_producer.send_state_messages(&state_metaes).await {
        Ok(_) => {
            info!(logger, "Successfully publish_state_messages");
        }
        Err(e) => {
            error!(logger, "Failed to publish_state_messages: {}", e);
        }
    }
    Ok(())
}
pub async fn publish_image_messages(
    message_producer: &dyn MessagePublisher,
    image_metaes: &[DicomImageMeta],
) -> Result<(), Whatever> {
    if image_metaes.is_empty() {
        return Ok(());
    }
    let logger = get_logger();
    let logger = logger.new(o!("thread" => "publish_image_messages"));
    match message_producer.send_image_messages(&image_metaes).await {
        Ok(_) => {
            info!(logger, "Successfully publish_image_messages");
        }
        Err(e) => {
            error!(logger, "Failed to publish_image_messages: {}", e);
        }
    }
    Ok(())
}
