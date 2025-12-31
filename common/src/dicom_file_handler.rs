use crate::dicom_utils::{get_bounder_string, get_date_value_dicom, get_tag_value};
use crate::message_sender_kafka::KafkaMessagePublisher;
use crate::storage_config::{StorageConfig, hash_uid};
use crate::utils;
use crate::utils::get_logger;
use crate::{server_config, storage_config};
use database::dicom_dbtype::{BoundedString, FixedLengthString};
use database::dicom_meta::{DicomStoreMeta, TransferStatus};
use dicom_dictionary_std::tags;
use dicom_encoding::TransferSyntaxIndex;
use dicom_encoding::snafu::{ResultExt, Whatever, whatever};
use dicom_object::{DefaultDicomObject, FileMetaTableBuilder, InMemDicomObject, OpenFileOptions};
use dicom_pixeldata::Transcode;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use slog::o;
use slog::{error, info};
use std::collections::HashSet;
use std::sync::LazyLock;
use uuid::Uuid;

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

pub async fn process_dicom_buffer(
    instance_buffer: &[u8],    //DICOM ByteStream
    tenant_id: &String,        //hospital  or tenant id , or department id
    ts: &String,               //Transfer Syntax UID
    sop_instance_uid: &String, //Current file's SOP Instance UID
    sop_class_uid: &String,    //Current file's SOP Class UID
    ip: String,  // source  IP address of the DICOM sender
    client_ae: String, // source  AE Title of the DICOM sender
    storage_config: &StorageConfig<'_>, // storage config
) -> Result<DicomStoreMeta, Whatever> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("wado-storescp"=>"process_dicom_file"));
    let obj = InMemDicomObject::read_dataset_with_ts(
        instance_buffer,
        TransferSyntaxRegistry.get(ts).unwrap(),
    )
    .whatever_context("failed to read DICOM data object")?;
    info!(logger, "DICOM data object read successfully");
    let pat_id = match get_bounder_string::<64>(&obj, tags::PATIENT_ID) {
        Some(v) => v,
        None => {
            whatever!("Missing PatientID or PatientID value length is exceeded 64 characters")
        }
    };

    let study_uid = match get_bounder_string::<64>(&obj, tags::STUDY_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing StudyID or StudyID value length is exceeded 64 characters")
        }
    };

    let series_uid = match get_bounder_string::<64>(&obj, tags::SERIES_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing SeriesID or SeriesID value length is exceeded 64 characters")
        }
    };

    let accession_number = get_bounder_string::<16>(&obj, tags::ACCESSION_NUMBER);

    let study_date = match get_date_value_dicom(&obj, tags::STUDY_DATE) {
        Some(v) => v,
        None => {
            whatever!("Missing STUDY_DATE or STUDY_DATE value is not invalid format YYYYMMDD")
        }
    };

    let frames = get_tag_value(tags::NUMBER_OF_FRAMES, &obj, 1);
    info!(
        logger,
        "TenantID:{} ,PatientID: {}, StudyUID: {},   StudyDate: {},  Frames: {}",
        tenant_id,
        pat_id,
        study_uid,
        study_date,
        frames
    );

    let file_meta = FileMetaTableBuilder::new()
        .media_storage_sop_class_uid(sop_class_uid)
        .media_storage_sop_instance_uid(sop_instance_uid)
        .transfer_syntax(ts)
        .build()
        .whatever_context("failed to build DICOM meta file information")?;
    let file_obj = obj.with_exact_meta(file_meta);

    let study_date_str = study_date.format("%Y%m%d").to_string();
    let dir_path = storage_config
        .make_series_dicom_dir(
            tenant_id,
            &*study_date_str,
            study_uid.as_str(),
            series_uid.as_str(),
            true,
        )
        .whatever_context(format!(
        "failed to get dicom series dir: tenant_id={}, study_date={}, study_uid={}, series_uid={}",
        tenant_id, study_date, study_uid, series_uid
    ))?;
    let study_uid_hash_v = hash_uid(study_uid.as_str());
    let series_uid_hash_v = hash_uid(series_uid.as_str());

    let file_path = storage_config::dicom_file_path(&dir_path, sop_instance_uid);

    info!(logger, "file path: {}", file_path);
    let final_ts = ts.to_string();
    let mut transcode_status = TransferStatus::NoNeedTransfer;
    if !JS_SUPPORTED_TS.contains(ts) {
        transcode_status = TransferStatus::NeedTransfer;

    } else {
        info!(logger, "not need transcode: {}", ts.to_string());
    }
    file_obj
        .write_to_file(&file_path)
        .whatever_context(format!(
            "not need transcode, save file to disk failed: {:?}",
            file_path
        ))?;
    let fsize = match std::fs::metadata(&file_path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0u64,
    };

    let uuid_v7 = Uuid::now_v7();
    let trace_uid = uuid_v7.to_string(); // 或直接用 format!("{}", uuid_v7)
    // 修改为
    let cdate = chrono::Local::now().naive_local();

    Ok(DicomStoreMeta {
        trace_id: FixedLengthString::<36>::make(trace_uid),
        worker_node_id: BoundedString::<64>::make_str("DICOM_STORE_SCP"),
        tenant_id: BoundedString::<64>::make_str(&tenant_id),
        patient_id: pat_id,
        study_uid,
        series_uid,
        sop_uid: BoundedString::<64>::make_str(&sop_instance_uid),
        file_path: BoundedString::<512>::make_str(&file_path),
        file_size: fsize as i64,
        transfer_syntax_uid: BoundedString::<64>::make_str(ts),
        target_ts: BoundedString::<64>::make_str(&final_ts),
        study_date,
        transfer_status: transcode_status,
        number_of_frames: frames,
        created_time: cdate,
        series_uid_hash: BoundedString::<20>::make_str(&series_uid_hash_v),
        study_uid_hash: BoundedString::<20>::make_str(&study_uid_hash_v),
        accession_number,
        source_ip: BoundedString::<24>::make_str(&ip),
        source_ae: BoundedString::<64>::make_str(&client_ae),
    })
}

pub async fn process_dicom_memobject(
    obj: &mut DefaultDicomObject,
    dicom_file_path: &String,
    tenant_id: &String,
    _storage_config: &StorageConfig<'_>,
) -> Result<DicomStoreMeta, Whatever> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("wado-server"=>"process_dicom_file_from_file"));

    info!(
        logger,
        "process_dicom_file_from_file object read successfully"
    );
    let pat_id = match get_bounder_string::<64>(&obj, tags::PATIENT_ID) {
        Some(v) => v,
        None => {
            whatever!("Missing PatientID or PatientID value length is exceeded 64 characters")
        }
    };

    let study_uid = match get_bounder_string::<64>(&obj, tags::STUDY_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing StudyID or StudyID value length is exceeded 64 characters")
        }
    };

    let series_uid = match get_bounder_string::<64>(&obj, tags::SERIES_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing SeriesID or SeriesID value length is exceeded 64 characters")
        }
    };
    let sop_uid = match get_bounder_string::<64>(&obj, tags::SOP_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing SOP_INSTANCE_UID or length is exceeded 64 characters")
        }
    };
    let transfer_syntax_uid = match get_bounder_string::<64>(&obj, tags::TRANSFER_SYNTAX_UID) {
        Some(v) => v,
        None => BoundedString::<64>::make_str(obj.meta().transfer_syntax()),
    };

    let accession_number = get_bounder_string::<16>(&obj, tags::ACCESSION_NUMBER);

    let study_date = match get_date_value_dicom(&obj, tags::STUDY_DATE) {
        Some(v) => v,
        None => {
            whatever!("Missing STUDY_DATE or STUDY_DATE value is not invalid format YYYYMMDD")
        }
    };

    let frames = get_tag_value(tags::NUMBER_OF_FRAMES, &obj, 1);
    info!(
        logger,
        "TenantID:{} ,PatientID: {}, StudyUID: {},   StudyDate: {},  Frames: {}",
        tenant_id,
        pat_id,
        study_uid,
        study_date,
        frames
    );




    let study_uid_hash_v = hash_uid(study_uid.as_str());
    let series_uid_hash_v = hash_uid(series_uid.as_str());
    let uuid_v7 = Uuid::now_v7();
    let trace_uid = uuid_v7.to_string(); // 或直接用 format!("{}", uuid_v7)
    let mut transcode_status = TransferStatus::NoNeedTransfer;
    let final_ts = transfer_syntax_uid.to_string();
    if !JS_SUPPORTED_TS.contains(transfer_syntax_uid.as_str()) {
        transcode_status = TransferStatus::NeedTransfer;
    }


    let fsize = match std::fs::metadata(&dicom_file_path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0u64,
    };
    // 修改为
    let cdate = chrono::Local::now().naive_local();

    Ok(DicomStoreMeta {
        trace_id: FixedLengthString::<36>::make(trace_uid),
        worker_node_id: BoundedString::<64>::make_str("STOW-RS"),
        tenant_id: BoundedString::<64>::make_str(&tenant_id),
        patient_id: pat_id,
        study_uid,
        series_uid,
        sop_uid,
        file_path: BoundedString::<512>::make_str(&dicom_file_path),
        file_size: fsize as i64,
        transfer_syntax_uid,
        target_ts: BoundedString::<64>::make_str(&final_ts),
        study_date,
        transfer_status: transcode_status,
        number_of_frames: frames,
        created_time: cdate,
        // 修改为使用 from_string 方法创建 BoundedString<20>
        series_uid_hash: BoundedString::<20>::make_str(&series_uid_hash_v),
        study_uid_hash: BoundedString::<20>::make_str(&study_uid_hash_v),
        accession_number,
        source_ip: BoundedString::<24>::make_str("127.0.0.1"),
        source_ae: BoundedString::<64>::make_str(&"STOW-RS-API"),
    })
}

pub async fn process_dicom_file_from_file(
    dicom_file_path: &String,
    tenant_id: &String,
    _storage_config: &StorageConfig<'_>,
) -> Result<DicomStoreMeta, Whatever> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("wado-server"=>"process_dicom_file_from_file"));

    // 读取元数据.如果不需要进行编码转换的话,不要读取PIXEL_DATA
    let obj = match OpenFileOptions::new()
        .read_until(tags::PIXEL_DATA)
        .open_file(&dicom_file_path)
    {
        Ok(obj) => obj,
        Err(e) => {
            whatever!("process_dicom_file_from_file open dicom file failed: {}", e)
        }
    };
    info!(
        logger,
        "process_dicom_file_from_file object read successfully"
    );
    let pat_id = match get_bounder_string::<64>(&obj, tags::PATIENT_ID) {
        Some(v) => v,
        None => {
            whatever!("Missing PatientID or PatientID value length is exceeded 64 characters")
        }
    };

    let study_uid = match get_bounder_string::<64>(&obj, tags::STUDY_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing StudyID or StudyID value length is exceeded 64 characters")
        }
    };

    let series_uid = match get_bounder_string::<64>(&obj, tags::SERIES_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing SeriesID or SeriesID value length is exceeded 64 characters")
        }
    };
    let sop_uid = match get_bounder_string::<64>(&obj, tags::SOP_INSTANCE_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing SOP_INSTANCE_UID or length is exceeded 64 characters")
        }
    };
    let transfer_syntax_uid = match get_bounder_string::<64>(&obj, tags::TRANSFER_SYNTAX_UID) {
        Some(v) => v,
        None => {
            whatever!("Missing TRANSFER_SYNTAX_UID or length is exceeded 64 characters")
        }
    };

    let accession_number = get_bounder_string::<16>(&obj, tags::ACCESSION_NUMBER);

    let study_date = match get_date_value_dicom(&obj, tags::STUDY_DATE) {
        Some(v) => v,
        None => {
            whatever!("Missing STUDY_DATE or STUDY_DATE value is not invalid format YYYYMMDD")
        }
    };

    let frames = get_tag_value(tags::NUMBER_OF_FRAMES, &obj, 1);
    info!(
        logger,
        "TenantID:{} ,PatientID: {}, StudyUID: {},   StudyDate: {},  Frames: {}",
        tenant_id,
        pat_id,
        study_uid,
        study_date,
        frames
    );
    let study_uid_hash_v = hash_uid(study_uid.as_str());
    let series_uid_hash_v = hash_uid(series_uid.as_str());
    let uuid_v7 = Uuid::now_v7();
    let trace_uid = uuid_v7.to_string(); // 或直接用 format!("{}", uuid_v7)
    let mut transcode_status = TransferStatus::NoNeedTransfer;
    let mut final_ts = transfer_syntax_uid.to_string();
    if !JS_SUPPORTED_TS.contains(transfer_syntax_uid.as_str()) {
        let target_ts = TransferSyntaxRegistry
            .get(JS_CHANGE_TO_TS.as_str())
            .unwrap();
        let mut trans_obj = match OpenFileOptions::new().open_file(&dicom_file_path) {
            Ok(obj) => obj,
            Err(e) => {
                whatever!("process_dicom_file_from_file open dicom file failed: {}", e)
            }
        };
        match trans_obj.transcode(target_ts) {
            Ok(_) => {
                final_ts = target_ts.uid().to_string();
                info!(
                    logger,
                    "transcode success: {} -> {}",
                    transfer_syntax_uid.to_string(),
                    final_ts
                );
                transcode_status = TransferStatus::Success;
            }
            Err(e) => {
                error!(logger, "transcode failed: {}", e);
                transcode_status = TransferStatus::Failed;
            }
        }
        match transcode_status {
            TransferStatus::Success => {
                obj.write_to_file(&dicom_file_path)
                    .whatever_context(format!(
                        "transcode success, biut save file to disk failed: {:?}",
                        dicom_file_path
                    ))?;
            }
            _ => {}
        }
    } else {
        info!(
            logger,
            "not need transcode: {}",
            transfer_syntax_uid.to_string()
        );
    }
    let fsize = match std::fs::metadata(&dicom_file_path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0u64,
    };
    // 修改为
    let cdate = chrono::Local::now().naive_local();

    Ok(DicomStoreMeta {
        trace_id: FixedLengthString::<36>::make(trace_uid),
        worker_node_id: BoundedString::<64>::make_str("STOW-RS"),
        tenant_id: BoundedString::<64>::make_str(&tenant_id),
        patient_id: pat_id,
        study_uid,
        series_uid,
        sop_uid,
        file_path: BoundedString::<512>::make_str(&dicom_file_path),
        file_size: fsize as i64,
        transfer_syntax_uid,
        target_ts: BoundedString::<64>::make_str(&final_ts),
        study_date,
        transfer_status: transcode_status,
        number_of_frames: frames,
        created_time: cdate,
        // 修改为使用 from_string 方法创建 BoundedString<20>
        series_uid_hash: BoundedString::<20>::make_str(&series_uid_hash_v),
        study_uid_hash: BoundedString::<20>::make_str(&study_uid_hash_v),
        accession_number,
        source_ip: BoundedString::<24>::make_str("127.0.0.1"),
        source_ae: BoundedString::<64>::make_str(&"STOW-RS-API"),
    })
}
/// Publishes DICOM metadata to Kafka topics
/// Files are saved to local disk regardless of transcoding success or failure
///
/// # Parameters
/// * `dicom_message_lists` - List of DICOM object metadata to be processed
/// * `storage_producer` - Kafka producer for extracting PatientInfo, StudyInfo, SeriesInfo as DICOM standard entities
/// * `log_producer` - Kafka producer for recording image receiving logs, facilitating subsequent efficiency statistics
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Returns Ok(()) if successful, otherwise returns an error
pub async fn classify_and_publish_dicom_messages(
    dicom_message_lists: &Vec<DicomStoreMeta>,
    storage_producer: &KafkaMessagePublisher,
    log_producer: &KafkaMessagePublisher,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("wado-storescp"=>"classify_and_publish_dicom_messages"));
    if dicom_message_lists.is_empty() {
        info!(logger, "Empty dicom message list, skip");
        return Ok(());
    }

    let message_count = dicom_message_lists.len();

    let topic_name = storage_producer.topic();

    match utils::publish_messages(storage_producer, &dicom_message_lists).await {
        Ok(_) => {
            info!(
                logger,
                "classify_and_publish_dicom_messages Successfully published {} supported messages to Kafka: {}",
                message_count,
                topic_name
            );
        }
        Err(e) => {
            error!(
                logger,
                "classify_and_publish_dicom_messages Failed to publish messages to Kafka: {},  topic: {}", e, topic_name
            );
        }
    }

    let log_topic_name = log_producer.topic();
    match utils::publish_messages(log_producer, &dicom_message_lists).await {
        Ok(_) => {
            info!(
                logger,
                "classify_and_publish_dicom_messages Successfully published {} messages to Kafka: {}",
                message_count,
                log_topic_name
            );
        }
        Err(e) => {
            error!(
                logger,
                "classify_and_publish_dicom_messages Failed to publish log messages to Kafka: {}, topic: {}", e, log_topic_name
            );
        }
    }

    Ok(())
}
