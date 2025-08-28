use crate::change_file_transfer::convert_ts_with_pixel_data;
use crate::cornerstonejs::SUPPORTED_TRANSFER_SYNTAXES;
use crate::database_entities::{
    DicomObjectMeta, ImageEntity, PatientEntity, SeriesEntity, StudyEntity,
};
use crate::database_provider::DbProvider;
use crate::database_provider_base::DbProviderBase;
use crate::message_sender::MessagePublisher;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::Whatever;
use dicom_object::ReadError;
use log::error;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use tracing::info;

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
// 设置日志记录，日志文件按大小滚动，保留最近7个文件
pub fn setup_logging(policy_name: &str) {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::append::rolling_file::RollingFileAppender;
    use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
    use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
    use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
    use log4rs::config::{Appender, Config, Logger, Root};
    use log4rs::encode::pattern::PatternEncoder;
    // 创建控制台appender
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} [{M}] {m}{n}",
        )))
        .build();

    // 创建基于文件大小的触发器 (例如: 5MB)
    let trigger = Box::new(SizeTrigger::new(5 * 1024 * 1024)); // 10MB

    // 创建删除旧文件的roller (保留最多7个文件，相当于最近7天)
    let roller = Box::new(DeleteRoller::new());

    // 创建复合策略
    let policy = Box::new(CompoundPolicy::new(trigger, roller));

    // 创建滚动文件appender
    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} [{M}] {m}{n}",
        )))
        .build(format!("./logs/{}.log", policy_name), policy)
        .unwrap();

    // 构建配置
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("app", LevelFilter::Debug))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .unwrap();

    // 初始化log4rs
    let _handle = log4rs::init_config(config).unwrap();
}

pub fn get_unique_tenant_ids(message: &[DicomObjectMeta]) -> Vec<String> {
    let tenant_ids: HashSet<String> = message.iter().map(|m| m.tenant_id.clone()).collect();

    tenant_ids.into_iter().collect()
}

// 获取消息组中所有不同的 tenant_id
pub async fn group_dicom_messages(
    messages: &[DicomObjectMeta],
) -> Result<
    (
        Vec<PatientEntity>,
        Vec<StudyEntity>,
        Vec<SeriesEntity>,
        Vec<ImageEntity>,
        Vec<DicomObjectMeta>,
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
    let mut failed_messages: Vec<DicomObjectMeta> = Vec::new();
    let mut failed_message_set: HashSet<&DicomObjectMeta> = HashSet::new(); // 用于跟踪已失败的消息

    {
        //-------------对所有的DICOM文件进行转码------
        for dcm_msg in messages {
            let support_ts =
                SUPPORTED_TRANSFER_SYNTAXES.contains(&dcm_msg.transfer_synatx_uid.as_str());
            if support_ts {
                continue;
            }

            let target_path = format!("./{}.dcm", dcm_msg.sop_uid);
            let src_file = dcm_msg.file_path.as_str();
            let src_sz = dcm_msg.file_size as usize;
            // 处理文件转换
            let conversion_result =
                convert_ts_with_pixel_data(src_file, src_sz, &target_path, true).await;
            if let Err(e) = conversion_result {
                tracing::error!("Change Dicom TransferSyntax To RLE Failed: {:?}", e);
                if !failed_message_set.contains(dcm_msg) {
                    failed_messages.push(dcm_msg.clone());
                    failed_message_set.insert(dcm_msg);
                }
                // 继续处理下一条消息
                continue;
            }
            // 转换成功，删除临时文件
            if let Err(remove_err) = fs::remove_file(&target_path) {
                tracing::error!(
                    "Failed to delete temporary file {}: {:?}",
                    target_path,
                    remove_err
                );
                // 即使删除失败也继续处理
            }
        }
    }

    for message in messages {
        // 不处理在 failed_messages 列表中的消息
        if failed_message_set.contains(message) {
            continue;
        }
        match dicom_object::OpenFileOptions::new()
            .read_until(tags::PIXEL_DATA)
            .open_file(&message.file_path)
        {
            Ok(dicom_obj) => {
                // 处理成功的DICOM对象
                // 提取 patient
                let key = format!("{}_{}", message.tenant_id, message.patient_id);
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
                                message.file_path,
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
                let key2 = format!("{}_{}", message.tenant_id, message.study_uid);
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
                            message.file_path,
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
                let key3 = format!("{}_{}", message.tenant_id, message.series_uid);
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
                            message.file_path,
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
                        sop_entity.transfer_syntax_uid = message.transfer_synatx_uid.clone();
                        image_entities.push(sop_entity);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to extract image entity from file '{}': {}",
                            message.file_path,
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

    Ok((
        patient_entities,
        study_entities,
        series_entities,
        image_entities,
        failed_messages,
    ))
}

// 处理 主要队列  topic_main 的消息
pub async fn process_storage_messages(
    messages_to_process: &[DicomObjectMeta],
    db_provider: &Arc<dyn DbProvider>,
) {
    if messages_to_process.is_empty() {
        return;
    }
    tracing::info!("Processing batch of {} messages", messages_to_process.len());
    let unique_tenant_ids = get_unique_tenant_ids(&messages_to_process);
    tracing::info!("Processing data for tenant IDs: {:?}", unique_tenant_ids);

    for tenant_id in unique_tenant_ids {
        let tenant_msg = messages_to_process
            .iter()
            .filter(|m| m.tenant_id == tenant_id)
            .cloned()
            .collect::<Vec<_>>();
        match group_dicom_messages(&tenant_msg).await {
            Ok((patients, studies, series, images, failed_messages)) => {
                // 保存失败的消息到数据库
                if !failed_messages.is_empty() {
                    if let Err(e) = db_provider.save_dicommeta_info(&failed_messages).await {
                        tracing::error!("Failed to save failed messages to database: {}", e);
                    }
                }

                // 持久化成功提取的数据到数据库
                if let Err(e) = db_provider
                    .persist_to_database(tenant_id.as_str(), &patients, &studies, &series, &images)
                    .await
                {
                    tracing::error!(
                        "Failed to persist DICOM data to database for tenant {}: {}",
                        tenant_id,
                        e
                    );
                    // 将所有消息标记为失败并保存到数据库
                    if let Err(save_err) = db_provider.save_dicommeta_info(&tenant_msg).await {
                        tracing::error!(
                            "Failed to save tenant messages to failed table: {}",
                            save_err
                        );
                    }
                    continue;
                }

                tracing::info!("Successfully processed data for tenant {}", tenant_id);
            }
            Err(e) => {
                tracing::error!(
                    "Failed to group DICOM messages for tenant {}: {}",
                    tenant_id,
                    e
                );
                // 当分组处理失败时，将所有消息保存到失败表中
                if let Err(save_err) = db_provider.save_dicommeta_info(&tenant_msg).await {
                    tracing::error!(
                        "Failed to save tenant messages to failed table after group failure: {}",
                        save_err
                    );
                }
                continue;
            }
        }
    }
}

// 发送消息到指定队列
pub async fn publish_messages(
    message_producer: &dyn MessagePublisher,
    dicom_message_lists: &[DicomObjectMeta],
) -> Result<(), Whatever> {
    if dicom_message_lists.is_empty() {
        return Ok(());
    }
    match message_producer
        .send_batch_messages(&dicom_message_lists)
        .await
    {
        Ok(_) => {
            info!("Successfully publish_messages");
        }
        Err(e) => {
            error!("Failed to publish_messages: {}", e);
        }
    }
    Ok(())
}
