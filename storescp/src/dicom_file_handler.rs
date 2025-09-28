use common::database_entities::DicomObjectMeta;
use common::dicom_utils::get_tag_value;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::{ResultExt, Whatever};
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use std::path::PathBuf;
use tracing::info;

pub(crate) async fn process_dicom_file(
    instance_buffer: &[u8],    //DICOM文件的字节数组或是二进制流
    out_dir: &str,             //存储文件的根目录, 例如 :/opt/dicomStore/
    tenant_id: &String,        //机构ID,或是医院ID, 用于区分多个医院.
    ts: &String,               //传输语法
    sop_instance_uid: &String, //当前文件的SOP实例ID
    lst: &mut Vec<DicomObjectMeta>,
) -> Result<(), Whatever> {
    let obj = InMemDicomObject::read_dataset_with_ts(
        instance_buffer,
        TransferSyntaxRegistry.get(ts).unwrap(),
    )
    .whatever_context("failed to read DICOM data object")?;
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

    let sop_uid = obj
        .element(tags::SOP_INSTANCE_UID)
        .whatever_context("Missing SOP_INSTANCE_UID")?
        .to_str()
        .whatever_context("could not retrieve SOP_INSTANCE_UID")?
        .trim_end_matches("\0")
        .to_string();

    let study_date = obj
        .element(tags::STUDY_DATE)
        .whatever_context("Missing STUDY_DATE")?
        .to_str()
        .whatever_context("could not retrieve STUDY_DATE")?
        .trim_end_matches("\0")
        .to_string();
    let modality = obj
        .element(tags::MODALITY)
        .whatever_context("Missing MODALITY")?
        .to_str()
        .whatever_context("could not retrieve MODALITY")?
        .trim_end_matches("\0")
        .to_string();

    let frames = get_tag_value(tags::NUMBER_OF_FRAMES, &obj, 1);
    info!(
        "Issur:{} ,PatientID: {}, StudyUID: {}, SeriesUID: {}, SopUID: {}, StudyDate: {}, Modality: {}, Frames: {}",
        tenant_id, pat_id, study_uid, series_uid, sop_uid, study_date, modality, frames
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
    let file_obj = obj.with_exact_meta(file_meta);
    let fp = out_dir.ends_with("/");
    let dir_path = match fp {
        true => {
            format!(
                "{}{}/{}/{}/{}",
                out_dir, tenant_id, pat_id, study_uid, series_uid
            )
        }
        false => {
            format!(
                "{}/{}/{}/{}/{}",
                out_dir, tenant_id, pat_id, study_uid, series_uid
            )
        }
    };

    info!("file path: {}", dir_path);

    let ok = std::fs::exists(&dir_path);
    match ok {
        Ok(false) => {
            std::fs::create_dir_all(&dir_path).unwrap_or_else(|_e| {
                info!("create directory failed: {}", dir_path);
            });
        }
        _ => {}
    }

    let file_path = format!(
        "{}/{}",
        dir_path,
        sop_instance_uid.trim_end_matches('\0').to_string() + ".dcm"
    );

    file_obj
        .write_to_file(&file_path)
        .whatever_context("write file failed")?;
    let fsize = std::fs::metadata(&file_path).unwrap().len();
    info!("Stored {}, {} with:{} bytes", ts, sop_instance_uid, fsize);

    // // 从新从磁盘读取DICOM文件, 确保文件已经完全写入磁盘.
    // let dicom_obj = dicom_object::OpenFileOptions::new()
    //     .read_until(tags::PIXEL_DATA)
    //     .open_file(&file_path)
    //     .whatever_context("open dicom file failed")?;

    // 修复后：
    let saved_path = PathBuf::from(file_path); // 此时可以安全转移所有权

    lst.push(DicomObjectMeta {
        tenant_id: tenant_id.to_string(),
        patient_id: pat_id.to_string(),
        study_uid,
        series_uid,
        sop_uid,
        file_path: saved_path.to_str().unwrap().to_string(),
        file_size: fsize as i64,
        transfer_synatx_uid: ts.to_string(),
        number_of_frames: frames,
        created_time: None,
        updated_time: None,
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
    storage_producer: &common::message_sender_kafka::KafkaMessagePublisher,
    change_producer: &common::message_sender_kafka::KafkaMessagePublisher,
    logger: &slog::Logger,
    queue_topic_main: &str,
    queue_topic_change: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 将 dicom_message_lists 按 transfer_syntax_uid 是否被 SUPPORTED_TRANSFER_SYNTAXES 支持分成两类
    let (supported_messages, unsupported_messages): (Vec<_>, Vec<_>) = dicom_message_lists
        .iter()
        .partition(|meta| {
            common::cornerstonejs::SUPPORTED_TRANSFER_SYNTAXES
                .contains(meta.transfer_synatx_uid.as_str())
        });
    
    // 将引用转换为拥有所有权的 Vec
    let supported_messages_owned: Vec<_> = supported_messages.into_iter().cloned().collect();
    let unsupported_messages_owned: Vec<_> = unsupported_messages.into_iter().cloned().collect();

    // 使用 storage_producer 发布受支持的消息
    if !supported_messages_owned.is_empty() {
        match common::utils::publish_messages(storage_producer, &supported_messages_owned).await {
            Ok(_) => {
                slog::info!(
                    logger,
                    "Successfully published {} supported messages to Kafka: {}",
                    supported_messages_owned.len(),
                    queue_topic_main
                );
            }
            Err(e) => {
                slog::error!(
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
                slog::info!(
                    logger,
                    "Successfully published {} unsupported messages to Kafka: {}",
                    unsupported_messages_owned.len(),
                    queue_topic_change
                );
            }
            Err(e) => {
                slog::error!(
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
