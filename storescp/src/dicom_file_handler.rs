use common::database_entities::DicomObjectMeta;
use common::dicom_utils::get_tag_value;
use common::message_sender_kafka::KafkaMessagePublisher;
use common::server_config;
use common::utils::get_logger;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::{ ResultExt, Whatever};
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_pixeldata::Transcode;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use slog::o;
use slog::{error, info};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

/// 校验 DICOM StudyDate 格式是否符合 YYYYMMDD 格式
fn validate_study_date_format(date_str: &str) -> Result<(), &'static str> {
    if date_str.len() != 8 {
        return Err("StudyDate must be exactly 8 characters long");
    }

    // 检查是否全部为数字
    if !date_str.chars().all(|c| c.is_ascii_digit()) {
        return Err("StudyDate must contain only digits");
    }

    // 提取年、月、日部分
    let year = &date_str[0..4];
    let month = &date_str[4..6];
    let day = &date_str[6..8];

    // 检查月份范围
    let month_val = month.parse::<u32>().map_err(|_| "Invalid month format")?;
    if month_val < 1 || month_val > 12 {
        return Err("Month must be between 01 and 12");
    }

    // 检查日期范围
    let day_val = day.parse::<u32>().map_err(|_| "Invalid day format")?;
    if day_val < 1 || day_val > 31 {
        return Err("Day must be between 01 and 31");
    }

    // 更严格的日期有效性检查
    let year_val = year.parse::<i32>().map_err(|_| "Invalid year format")?;
    if !is_valid_date(year_val, month_val, day_val) {
        return Err("Invalid date");
    }

    Ok(())
}

/// 检查给定的年月日是否构成有效日期
fn is_valid_date(year: i32, month: u32, day: u32) -> bool {
    if month == 0 || month > 12 || day == 0 || day > 31 {
        return false;
    }

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29 // 闰年
            } else {
                28 // 平年
            }
        }
        _ => return false,
    };

    day <= days_in_month
}
static JS_SUPPORTED_TS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    // 在这里初始化，可以从配置文件读取
    // load_config 是INIT_ONCE 封装的, 不会重新加载
    let config = server_config::load_config().unwrap();
    config
        .dicom_store_scp
        .cornerstonejs_supported_transfer_syntax
        .iter()
        .cloned()
        .collect()
});

static JS_CHANGE_TO_TS: LazyLock<String> = LazyLock::new(|| {
    // 在这里初始化，可以从配置文件读取
    // load_config 是INIT_ONCE 封装的, 不会重新加载
    let config = server_config::load_config().unwrap();
    config.dicom_store_scp.unsupported_ts_change_to.clone()
});

pub(crate) async fn process_dicom_file(
    instance_buffer: &[u8],    //DICOM文件的字节数组或是二进制流
    tenant_id: &String,        //机构ID,或是医院ID, 用于区分多个医院.
    ts: &String,               //传输语法
    sop_instance_uid: &String, //当前文件的SOP实例ID
    lst: &mut Vec<DicomObjectMeta>,
) -> Result<(), Whatever> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("storescp"=>"process_dicom_file"));
    let obj = InMemDicomObject::read_dataset_with_ts(
        instance_buffer,
        TransferSyntaxRegistry.get(ts).unwrap(),
    )
    .whatever_context("failed to read DICOM data object")?;
    info!(logger, "DICOM data object read successfully");
    let pat_id = obj
        .element(tags::PATIENT_ID)
        .whatever_context("Missing PatientID")?
        .to_str()
        .whatever_context("could not retrieve PatientID")?
        .trim_end_matches("\0")
        .to_string();

    let study_uid = obj
        .element(tags::STUDY_INSTANCE_UID)
        .whatever_context("Missing StudyID")?
        .to_str()
        .whatever_context("could not retrieve STUDY_INSTANCE_UID")?
        .trim_end_matches("\0")
        .to_string();

    let series_uid = obj
        .element(tags::SERIES_INSTANCE_UID)
        .whatever_context("Missing SeriesID")?
        .to_str()
        .whatever_context("could not retrieve SERIES_INSTANCE_UID")?
        .trim_end_matches("\0")
        .to_string();

    let accession_number = obj
        .element(tags::ACCESSION_NUMBER)
        .whatever_context("Missing ACCESSION_NUMBER")?
        .to_str()
        .whatever_context("could not retrieve ACCESSION_NUMBER")?
        .trim_end_matches("\0")
        .to_string();

    let study_date = obj
        .element(tags::STUDY_DATE)
        .whatever_context("Missing STUDY_DATE")?
        .to_str()
        .whatever_context("could not retrieve STUDY_DATE")?
        .trim_end_matches("\0")
        .to_string();
    // 校验StudyDate格式是否正确. YYYYMMDD 格式
    validate_study_date_format(&study_date).whatever_context("Invalid StudyDate format")?;
    let modality = obj
        .element(tags::MODALITY)
        .whatever_context("Missing MODALITY")?
        .to_str()
        .whatever_context("could not retrieve MODALITY")?
        .trim_end_matches("\0")
        .to_string();

    let frames = get_tag_value(tags::NUMBER_OF_FRAMES, &obj, 1);
    info!(
        logger,
        "TenantID:{} ,PatientID: {}, StudyUID: {}, AccessionNumber: {}, StudyDate: {}, Modality: {}, Frames: {}",
        tenant_id,
        pat_id,
        study_uid,
        accession_number,
        study_date,
        modality,
        frames
    );

    let file_meta = FileMetaTableBuilder::new()
        .media_storage_sop_class_uid(
            obj.element(tags::SOP_CLASS_UID)
                .whatever_context("missing SOP Class UID")?
                .to_str()
                .whatever_context("could not retrieve SOP Class UID")?,
        )
        .media_storage_sop_instance_uid(
            obj.element(tags::SOP_INSTANCE_UID)
                .whatever_context("missing SOP Instance UID")?
                .to_str()
                .whatever_context("missing SOP Instance UID")?,
        )
        .transfer_syntax(ts)
        .build()
        .whatever_context("failed to build DICOM meta file information")?;
    let mut file_obj = obj.with_exact_meta(file_meta);

    let (study_uid_hash_v, series_uid_hash_v, dir_path) =
        server_config::dicom_series_dir(tenant_id, &study_date, &study_uid, &series_uid, true)
            .whatever_context(format!(
        "failed to get dicom series dir: tenant_id={}, study_date={}, study_uid={}, series_uid={}",
        tenant_id, study_date, study_uid, series_uid
    ))?;

    let file_path = format!("{}/{}.dcm", dir_path, sop_instance_uid);

    info!(logger, "file path: {}", file_path);
    let mut final_ts = ts.to_string();
    if !JS_SUPPORTED_TS.contains(ts) {
        let target_ts = TransferSyntaxRegistry
            .get(JS_CHANGE_TO_TS.as_str())
            .unwrap();
        match file_obj.transcode(target_ts) {
            Ok(_) => {
                final_ts = target_ts.uid().to_string();
                info!(
                    logger,
                    "transcode success: {} -> {}",
                    ts.to_string(),
                    final_ts
                );
            }
            Err(e) => {
                error!(logger, "transcode failed: {}", e);
            }
        }
    } else {
        info!(logger, "not need transcode: {}", ts.to_string());
    }
    file_obj
        .write_to_file(&file_path)
        .whatever_context(format!(
            "not need transcode, save file to disk failed: {:?}",
            file_path
        ))?;
    let fsize = std::fs::metadata(&file_path).unwrap().len();
    // 修复后：
    let saved_path = PathBuf::from(file_path); // 此时可以安全转移所有权

    lst.push(DicomObjectMeta {
        tenant_id: tenant_id.to_string(),
        patient_id: pat_id.to_string(),
        study_uid: study_uid.to_string(),
        series_uid: series_uid.to_string(),
        sop_uid: sop_instance_uid.to_string(),
        file_path: String::from(saved_path.to_str().unwrap()),
        file_size: fsize as i64,
        transfer_syntax_uid: final_ts.to_string(),
        number_of_frames: frames,
        created_time: None,
        updated_time: None,
        series_uid_hash: series_uid_hash_v,
        study_uid_hash: study_uid_hash_v,
    });
    Ok(())
}

/// 根据传输语法支持情况将 DICOM 消息列表分类并分别发布到不同的 Kafka 主题
///
/// # 参数
/// * `dicom_message_lists` - 需要处理的 DICOM 对象元数据列表
/// * `storage_producer` - 用于发布受支持传输语法消息的 Kafka 生产者
/// * `change_producer` - 用于发布不受支持传输语法消息的 Kafka 生产者
/// * `logger` - 日志记录器
/// * `queue_topic_main` - 主题名称（受支持的传输语法）
/// * `queue_topic_change` - 主题名称（不受支持的传输语法）
pub(crate) async fn classify_and_publish_dicom_messages(
    dicom_message_lists: &Vec<DicomObjectMeta>,
    storage_producer: &KafkaMessagePublisher,
    change_producer: &KafkaMessagePublisher,
    queue_topic_main: &str,
    queue_topic_change: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("storescp"=>"classify_and_publish_dicom_messages"));
    // 将 dicom_message_lists 按 transfer_syntax_uid 是否被 SUPPORTED_TRANSFER_SYNTAXES 支持分成两类
    let (supported_messages, unsupported_messages): (Vec<_>, Vec<_>) = dicom_message_lists
        .iter()
        .partition(|meta| JS_SUPPORTED_TS.contains(meta.transfer_syntax_uid.as_str()));

    // 将引用转换为拥有所有权的 Vec
    let supported_messages_owned: Vec<_> = supported_messages.into_iter().cloned().collect();
    let unsupported_messages_owned: Vec<_> = unsupported_messages.into_iter().cloned().collect();

    // 使用 storage_producer 发布受支持的消息
    if !supported_messages_owned.is_empty() {
        match common::utils::publish_messages(storage_producer, &supported_messages_owned).await {
            Ok(_) => {
                info!(
                    logger,
                    "Successfully published {} supported messages to Kafka: {}",
                    supported_messages_owned.len(),
                    queue_topic_main
                );
            }
            Err(e) => {
                error!(
                    logger,
                    "Failed to publish supported messages to Kafka: {}, topic: {}",
                    e,
                    queue_topic_main
                );
            }
        }
    }

    // 使用 change_producer 发布不受支持的消息
    if !unsupported_messages_owned.is_empty() {
        match common::utils::publish_messages(change_producer, &unsupported_messages_owned).await {
            Ok(_) => {
                info!(
                    logger,
                    "Successfully published {} unsupported messages to Kafka: {}",
                    unsupported_messages_owned.len(),
                    queue_topic_change
                );
            }
            Err(e) => {
                error!(
                    logger,
                    "Failed to publish unsupported messages to Kafka: {}, topic: {}",
                    e,
                    queue_topic_change
                );
            }
        }
    }

    Ok(())
}
