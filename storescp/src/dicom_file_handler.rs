use common::database_entities::DicomObjectMeta;
use common::dicom_utils::get_tag_value;
use common::message_sender::MessagePublisher;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::{ResultExt, Whatever};
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use std::path::PathBuf;
use tracing::info;
use tracing::log::error;

pub(crate) async fn process_dicom_file(
    instance_buffer: &[u8],    //DICOM文件的字节数组或是二进制流
    out_dir: &str,             //存储文件的根目录, 例如 :/opt/dicomStore/
    issue_patient_id: &String, // 机构ID,或是医院ID, 用于区分多个医院.
    ts: &String,               // 传输语法
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
    let frames = get_tag_value(tags::NUMBER_OF_FRAMES, &obj, 1);
    info!(
        "Issur:{} ,PatientID: {}, StudyUID: {}, SeriesUID: {}, SopUID: {}",
        issue_patient_id, pat_id, study_uid, series_uid, sop_uid
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
                out_dir, issue_patient_id, pat_id, study_uid, series_uid
            )
        }
        false => {
            format!(
                "{}/{}/{}/{}/{}",
                out_dir, issue_patient_id, pat_id, study_uid, series_uid
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

    file_obj.write_to_file(&file_path).whatever_context("write file failed")?;
    let fsize  = std::fs::metadata(&file_path).unwrap().len();
    info!("Stored {}, {} with:{} bytes", ts, sop_instance_uid,fsize);

    // // 从新从磁盘读取DICOM文件, 确保文件已经完全写入磁盘.
    // let dicom_obj = dicom_object::OpenFileOptions::new()
    //     .read_until(tags::PIXEL_DATA)
    //     .open_file(&file_path)
    //     .whatever_context("open dicom file failed")?;

    // 修复后：
    let saved_path = PathBuf::from(file_path); // 此时可以安全转移所有权

    lst.push(DicomObjectMeta {
        tenant_id: issue_patient_id.to_string(),
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

// 发布消息到消息队列
// parameters:
// storage_producer: 用于存储的消息生产者
// multi_frames_producer: 用于多帧图像处理的消息生产者
// chgts_producer: 用于传输语法转换的消息生产
// dicom_message_lists: 包含DICOM对象元数据的列表

pub(crate) async fn publish_messages(
    storage_producer: &dyn MessagePublisher,
    multi_frames_producer: Option<&dyn MessagePublisher>,
    chage_ts_producer: Option<&dyn MessagePublisher>,
    dicom_message_lists: &Vec<DicomObjectMeta>,
) -> Result<(), Whatever> {
    if dicom_message_lists.is_empty() {
        return Ok(());
    }
    match storage_producer
        .send_batch_messages(&dicom_message_lists)
        .await
    {
        Ok(_) => {
            info!("Successfully sent messages to Kafka");
        }
        Err(e) => {
            error!("Failed to send messages to Kafka: {}", e);
        }
    }

    if multi_frames_producer.is_none() && chage_ts_producer.is_none() {
        return Ok(());
    }

    match chage_ts_producer {
        Some(producer) => {
            info!("chgts_kafka_producer is not None");
            //----需要创建一个KafkaProducer单独处理多帧图像
            //----多帧会进行CHGTS转换
            let chgts_messages: Vec<_> = dicom_message_lists
                .iter()
                .filter(|msg| {
                    msg.number_of_frames == 1
                        && !common::cornerstonejs::SUPPORTED_TRANSFER_SYNTAXES
                            .contains(&msg.transfer_synatx_uid.as_str())
                })
                .cloned()
                .collect();
            if chgts_messages.len() > 0 {
                match producer.send_batch_messages(&chgts_messages).await {
                    Ok(_) => {
                        info!("Successfully sent messages to topic-change transfersynatx ");
                    }
                    Err(e) => {
                        error!(
                            "Failed to send messages to topic-change transfersynatx : {}",
                            e
                        );
                    }
                }
            }
        }
        None => {
            info!("chgts_kafka_producer is None");
        }
    }

    match multi_frames_producer {
        Some(producer) => {
            info!("multi_frames_kafka_producer is not None");
            //----需要创建一个KafkaProducer单独处理多帧图像
            //----多帧会对DICOM图像进行传输语法转换.
            let multi_frame_messages: Vec<_> = dicom_message_lists
                .iter()
                .filter(|msg| msg.number_of_frames > 1)
                .cloned()
                .collect();

            if multi_frame_messages.len() > 0 {
                match producer.send_batch_messages(&multi_frame_messages).await {
                    Ok(_) => {
                        info!("Successfully sent messages to topic_multi_frames");
                    }
                    Err(e) => {
                        error!("Failed to send messages to topic_multi_frames: {}", e);
                    }
                }
            }
        }
        None => {
            info!("multi_frames_kafka_producer is None");
        }
    }

    // 添加返回语句
    Ok(())
}
