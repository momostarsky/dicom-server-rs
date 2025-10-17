use crate::database_entities::{ImageEntity, PatientEntity, SeriesEntity, StudyEntity};
use crate::database_provider::DbProvider;
use crate::database_provider_base::DbProviderBase;
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
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, OnceLock};

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
        let study_uid =Option::from(message.study_uid.as_str());
        match dicom_object::OpenFileOptions::new()
            .charset_override(CharacterSetOverride::AnyVr)
            .read_until(tags::PIXEL_DATA)
            .open_file(String::from(message.file_path.as_str()))
        {
            Ok(dicom_obj) => {
                let state_meta = make_state_info(message.tenant_id.as_str(), &dicom_obj,  study_uid);
                let image_entity = make_image_info(message.tenant_id.as_str(), &dicom_obj);
                if state_meta.is_ok() && image_entity.is_ok() {
                    state_metas.push(state_meta.unwrap());
                    image_entities.push(image_entity.unwrap());
                } else {
                    error!(
                        logger,
                        "Failed to extract state or image entity from file: {} , message: {:?}",
                        message.file_path.as_str(),
                        message
                    );
                }
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

// 获取消息组中所有不同的 tenant_id
pub async fn group_dicom_messages(
    messages: &[DicomStoreMeta],
) -> Result<
    (
        Vec<PatientEntity>,
        Vec<StudyEntity>,
        Vec<SeriesEntity>,
        Vec<ImageEntity>,
        Vec<DicomStoreMeta>,
    ),
    ReadError,
> {
    let mut patient_groups: HashMap<String, bool> = HashMap::new();
    let mut study_groups: HashMap<String, bool> = HashMap::new();
    let mut series_groups: HashMap<String, bool> = HashMap::new();

    let mut patient_entities: Vec<PatientEntity> = Vec::new();
    let mut study_entities: Vec<StudyEntity> = Vec::new();
    let mut series_entities: Vec<SeriesEntity> = Vec::new();
    let mut image_entities: Vec<ImageEntity> = Vec::new();
    let mut failed_messages: Vec<DicomStoreMeta> = Vec::new();
    let mut failed_message_set: HashSet<&DicomStoreMeta> = HashSet::new(); // 用于跟踪已失败的消息

    for message in messages {
        match dicom_object::OpenFileOptions::new()
            .charset_override(CharacterSetOverride::AnyVr)
            .read_until(tags::PIXEL_DATA)
            .open_file(String::from(message.file_path.as_str()))
        {
            Ok(dicom_obj) => {
                // 处理成功的DICOM对象
                // 提取 patient
                let key = format!(
                    "{}_{}",
                    message.tenant_id.as_str(),
                    message.patient_id.as_str()
                );
                if !patient_groups.contains_key(key.as_str()) {
                    match DbProviderBase::extract_patient_entity(
                        message.tenant_id.as_str(),
                        &dicom_obj,
                    ) {
                        Ok(patient_entity) => {
                            patient_entities.push(patient_entity);
                            patient_groups.insert(key.clone(), true);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to extract patient entity from file '{}': {}",
                                message.file_path.as_str(),
                                e
                            );
                            if !failed_message_set.contains(message) {
                                failed_messages.push(message.clone());
                                failed_message_set.insert(message);
                            }
                            continue;
                        }
                    }
                }

                // 提取 study
                let key2 = format!(
                    "{}_{}",
                    message.tenant_id.as_str(),
                    message.study_uid.as_str()
                );
                match DbProviderBase::extract_study_entity(
                    message.tenant_id.as_str(),
                    &dicom_obj,
                    message.patient_id.as_str(),
                ) {
                    Ok(study_entity) => {
                        study_entities.push(study_entity);
                        study_groups.insert(key2.clone(), true);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to extract study entity from file '{}': {}",
                            message.file_path.as_str(),
                            e
                        );
                        if !failed_message_set.contains(message) {
                            failed_messages.push(message.clone());
                            failed_message_set.insert(message);
                        }
                        continue;
                    }
                }
                // 提取 series
                let key3 = format!(
                    "{}_{}",
                    message.tenant_id.as_str(),
                    message.series_uid.as_str()
                );
                match DbProviderBase::extract_series_entity(
                    message.tenant_id.as_str(),
                    &dicom_obj,
                    message.study_uid.as_str(),
                ) {
                    Ok(series_entity) => {
                        series_entities.push(series_entity);
                        series_groups.insert(key3.clone(), true);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to extract series entity from file '{}': {}",
                            message.file_path.as_str(),
                            e
                        );
                        if !failed_message_set.contains(message) {
                            failed_messages.push(message.clone());
                            failed_message_set.insert(message);
                        }
                        continue;
                    }
                }
                if !patient_groups.contains_key(key.as_str()) {
                    continue;
                }

                match DbProviderBase::extract_image_entity(
                    message.tenant_id.as_str(),
                    &dicom_obj,
                    message.patient_id.as_str(),
                    message.study_uid.as_str(),
                    message.series_uid.as_str(),
                ) {
                    Ok(mut sop_entity) => {
                        sop_entity.space_size = Some(message.file_size);
                        sop_entity.transfer_syntax_uid =
                            String::from(message.transfer_syntax_uid.as_str());
                        image_entities.push(sop_entity);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to extract image entity from file '{}': {}",
                            message.file_path.as_str(),
                            e
                        );
                        if !failed_message_set.contains(message) {
                            failed_messages.push(message.clone());
                            failed_message_set.insert(message);
                        }
                        continue;
                    }
                }
            }
            Err(_) => {
                // 记录错误但继续处理其他文件
                if !failed_message_set.contains(message) {
                    failed_messages.push(message.clone());
                    failed_message_set.insert(message);
                }
            }
        }
    }

    //TODO: 对StudyUIDHash和SeriesUIDHash和 DicomObjectMeta的StudyUIDHash和SeriesUIDHash进行比对.



    Ok((
        patient_entities,
        study_entities,
        series_entities,
        image_entities,
        failed_messages,
    ))
}

async fn write_failed_messages_to_file(
    failed_messages: &[DicomStoreMeta],
) -> Result<(), Box<dyn std::error::Error>> {
    // 以追加模式打开文件
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./consumer_failed.json")?;

    // 为每条失败消息写入一行JSON
    for message in failed_messages {
        let json_line = serde_json::to_string(message)?;
        writeln!(file, "{}", json_line)?;
    }

    Ok(())
}
// 处理 主要队列  topic_main 的消息
pub async fn process_storage_messages(
    messages_to_process: &[DicomStoreMeta],
    db_provider: &Arc<dyn DbProvider>,
) -> Result<(), Box<dyn std::error::Error>> {
    if messages_to_process.is_empty() {
        return Ok(());
    }
    let logger = get_logger();
    let logger = logger.new(o!("thread" => "process_storage_messages"));
    info!(
        logger,
        "XXX Processing batch of {} messages",
        messages_to_process.len()
    );
    let unique_tenant_ids = get_unique_tenant_ids(&messages_to_process);
    info!(
        logger,
        "Processing data for tenant IDs: {:?}", unique_tenant_ids
    );

    for tenant_id in unique_tenant_ids {
        let tenant_msg = messages_to_process
            .iter()
            .filter(|m| m.tenant_id.as_str() == tenant_id.as_str())
            .cloned()
            .collect::<Vec<_>>();
        match group_dicom_messages(&tenant_msg).await {
            Ok((patients, studies, series, images, failed_messages)) => {
                // 保存失败的消息到数据库
                if !failed_messages.is_empty() {
                    // 将失败的消息追加写入文件 ./failed.json
                    match write_failed_messages_to_file(&failed_messages).await {
                        Ok(_) => {
                            info!(
                                logger,
                                "Successfully wrote {} failed messages to failed.json",
                                failed_messages.len()
                            );
                        }
                        Err(e) => {
                            error!(logger, "Failed to write failed messages to file: {}", e);
                        }
                    }
                }

                // 持久化成功提取的数据到数据库
                if let Err(e) = db_provider
                    .persist_to_database(tenant_id.as_str(), &patients, &studies, &series, &images)
                    .await
                {
                    error!(
                        logger,
                        "Failed to persist DICOM data to database for tenant {}: {}", tenant_id, e
                    );
                    continue;
                }

                info!(
                    logger,
                    "Successfully processed data for tenant {}", tenant_id
                );
            }
            Err(e) => {
                error!(
                    logger,
                    "Failed to group DICOM messages for tenant {}: {}", tenant_id, e
                );
                continue;
            }
        }
    }
    Ok(())
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
