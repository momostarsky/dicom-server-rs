use common::dicom_object_meta::{DicomStoreMeta, TransferStatus};
use common::dicom_utils::get_tag_value;
use common::message_sender_kafka::KafkaMessagePublisher;
use common::server_config;
use common::utils::get_logger;
use dicom_core::chrono::Utc;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::{ResultExt, Whatever};
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_pixeldata::Transcode;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use slog::o;
use slog::{error, info};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;
use uuid::Uuid;

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
    ip: String,
    client_ae: String,
) -> Result<DicomStoreMeta, Whatever> {
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

    let file_path = server_config::dicom_file_path(&dir_path, sop_instance_uid);

    info!(logger, "file path: {}", file_path);
    let mut final_ts = ts.to_string();
    let mut transcode_status = TransferStatus::NoNeedTransfer;
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
                transcode_status = TransferStatus::Success;
            }
            Err(e) => {
                error!(logger, "transcode failed: {}", e);
                transcode_status = TransferStatus::Failed;
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
    let uuid_v7 = Uuid::now_v7();
    let trace_uid = uuid_v7.to_string(); // 或直接用 format!("{}", uuid_v7)
    // 修改为
    let cdate = chrono::Local::now().naive_local(); 
    Ok(DicomStoreMeta {
        trace_id: common::string_ext::UuidString::try_from(trace_uid)
            .with_whatever_context(|err| format!("Failed to create trace_id: {}", err))?,
        worker_node_id: common::string_ext::BoundedString::try_from("DICOM_STORE_SCP")
            .with_whatever_context(|err| format!("Failed to create worker_node_id: {}", err))?,
        tenant_id: common::string_ext::BoundedString::try_from(tenant_id)
            .with_whatever_context(|err| format!("Failed to create tenant_id: {}", err))?,
        patient_id: common::string_ext::BoundedString::try_from(pat_id)
            .with_whatever_context(|err| format!("Failed to create patient_id: {}", err))?,
        study_uid: common::string_ext::SopUidString::try_from(study_uid)
            .with_whatever_context(|err| format!("Failed to create study_uid: {}", err))?,
        series_uid: common::string_ext::SopUidString::try_from(series_uid)
            .with_whatever_context(|err| format!("Failed to create series_uid: {}", err))?,
        sop_uid: common::string_ext::SopUidString::try_from(sop_instance_uid)
            .with_whatever_context(|err| format!("Failed to create sop_uid: {}", err))?,
        file_path: common::string_ext::BoundedString::try_from(saved_path.to_str().unwrap())
            .with_whatever_context(|err| format!("Failed to create file_path: {}", err))?,
        file_size: fsize,
        transfer_syntax_uid: common::string_ext::SopUidString::try_from(ts).with_whatever_context(
            |err| format!("Failed to create transfer_syntax_uid: {}", err),
        )?,
        target_ts: common::string_ext::SopUidString::try_from(final_ts)
            .with_whatever_context(|err| format!("Failed to create target_ts: {}", err))?,
        study_date: common::string_ext::DicomDateString::try_from(study_date)
            .with_whatever_context(|err| format!("Failed to create study_date: {}", err))?,
        transfer_status: transcode_status,
        number_of_frames: frames,
        created_time: cdate,
        series_uid_hash: series_uid_hash_v,
        study_uid_hash: study_uid_hash_v,
        accession_number: common::string_ext::BoundedString::try_from(accession_number)
            .with_whatever_context(|err| format!("Failed to create accession_number: {}", err))?,
        source_ip: common::string_ext::BoundedString::try_from(ip)
            .with_whatever_context(|err| format!("Failed to create source_ip: {}", err))?,
        source_ae: common::string_ext::BoundedString::try_from(client_ae)
            .with_whatever_context(|err| format!("Failed to create source_ae: {}", err))?,
    })
}

///  无论转码成功与否，均会保存文件到本地磁盘
///
/// # 参数
/// * `dicom_message_lists` - 需要处理的 DICOM 对象元数据列表
/// * `storage_producer` - 用于发布受支持传输语法消息的 Kafka 生产者
/// * `log_producer` - 用于发布不受支持传输语法消息的 Kafka 生产者
/// * `logger` - 日志记录器
/// * `queue_topic_main` - 主题名称（用于storage_consumer）
/// * `queue_topic_log` - 主题名称（用于日志提取）
pub(crate) async fn classify_and_publish_dicom_messages(
    dicom_message_lists: &Vec<DicomStoreMeta>,
    storage_producer: &KafkaMessagePublisher,
    log_producer: &KafkaMessagePublisher,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("storescp"=>"classify_and_publish_dicom_messages"));
    if dicom_message_lists.is_empty() {
        info!(logger, "Empty dicom message list, skip");
        return Ok(());
    }

    let topic_name = storage_producer.topic();

    match common::utils::publish_messages(storage_producer, &dicom_message_lists).await {
        Ok(_) => {
            info!(
                logger,
                "Successfully published {} supported messages to Kafka: {}",
                dicom_message_lists.len(),
                topic_name
            );
        }
        Err(e) => {
            error!(
                logger,
                "Failed to publish messages to Kafka: {}, topic: {}", e, topic_name
            );
        }
    }

    let log_topic_name = log_producer.topic();
    match common::utils::publish_messages(log_producer, &dicom_message_lists).await {
        Ok(_) => {
            info!(
                logger,
                "Successfully published {} messages to Kafka: {}",
                dicom_message_lists.len(),
                log_topic_name
            );
        }
        Err(e) => {
            error!(
                logger,
                "Failed to publish log messages to Kafka: {}, topic: {}", e, log_topic_name
            );
        }
    }

    Ok(())
}
