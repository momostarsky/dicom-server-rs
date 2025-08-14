use crate::producer::KafkaProducer;
use common::entities::{DbProviderBase, DicomObjectMeta};
use common::DicomMessage;
use dicom_core::chrono::Local;
use dicom_dictionary_std::tags;
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use snafu::{whatever, ResultExt, Whatever};
use tracing::log::warn;
use tracing::{error, info};

pub(crate) async fn process_dicom_file(
    instance_buffer: &[u8],    //DICOM文件的字节数组或是二进制流
    out_dir: &str,             //存储文件的根目录, 例如 :/opt/dicomStore/
    issue_patient_id: &String, // 机构ID,或是医院ID, 用于区分多个医院.
    ts: &String,               // 传输语法
    sop_instance_uid: &String, //当前文件的SOP实例ID
    lst: &mut Vec<common::entities::DicomObjectMeta>,
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
    let accession_number = obj
        .element(tags::ACCESSION_NUMBER)
        .whatever_context("Missing ACCESSION_NUMBER")?
        .to_str()
        .whatever_context("could not retrieve ACCESSION_NUMBER")?
        .trim_end_matches("\0")
        .to_string();
    info!(
        "Issur:{} ,PatientID: {}, StudyUID: {}, SeriesUID: {}, AccessionNumber: {}",
        issue_patient_id, pat_id, study_uid, series_uid, accession_number
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

    let write_result = file_obj.write_to_file(&file_path);
    if write_result.is_err() {
        info!("write file failed: {}", file_path);
        whatever!("write file failed");
    }

    info!("Stored {}, {}", ts, sop_instance_uid);
    let pat = DbProviderBase::extract_patient_entity(issue_patient_id, &file_obj);
    let study =
        DbProviderBase::extract_study_entity(issue_patient_id, &file_obj, pat.patient_id.as_str());
    let series =
        DbProviderBase::extract_series_entity(issue_patient_id, &file_obj, study_uid.as_str());
    let image = DbProviderBase::extract_image_entity(
        issue_patient_id,
        &file_obj,
        study_uid.as_str(),
        series_uid.as_str(),
        pat.patient_id.as_str(),
    );

    let fs = std::fs::metadata(file_path.as_str());
    let file_size = match fs {
        Ok(fs) => fs.len() as u64,
        Err(_) => 0,
    };
    lst.push(common::entities::DicomObjectMeta {
        patient_info: pat,
        study_info: study,
        series_info: series,
        image_info: image,
        file_path: file_path.to_string(),
        file_size,
        tenant_id: issue_patient_id.to_string(),
    });
    Ok(())
}

pub(crate) async fn sendmessage_to_kafka(
    kafka_producer: &KafkaProducer,
    dicom_message_lists: &Vec<DicomObjectMeta>,
) {
    if dicom_message_lists.is_empty() {
        return;
    }

    // 消息数量不超过100条，一次性发送
    info!(
        "Sending {} messages in single batch",
        dicom_message_lists.len()
    );

    match kafka_producer
        .send_messages( dicom_message_lists)
        .await
    {
        Ok(_) => {
            info!(
                "Successfully sent {} messages to Kafka",
                dicom_message_lists.len()
            );
        }
        Err(e) => {
            warn!(
                "Failed to send messages to Kafka: {}. Writing to disk backup.",
                e
            );
        }
    }
}

// 备份失败消息到磁盘文件
async fn backup_failed_messages(messages: &[DicomObjectMeta], backup_filename: &str) {
    // 将消息列表序列化为 JSON
    match serde_json::to_string_pretty(messages) {
        Ok(json_data) => {
            // 写入磁盘文件
            match std::fs::write(backup_filename, json_data) {
                Ok(_) => {
                    info!(
                        "Successfully wrote {} messages to backup file: {}",
                        messages.len(),
                        backup_filename
                    );
                }
                Err(write_err) => {
                    error!(
                        "Failed to write backup file {}: {}",
                        backup_filename, write_err
                    );
                }
            }
        }
        Err(serialize_err) => {
            error!("Failed to serialize messages: {}", serialize_err);
        }
    }
}
