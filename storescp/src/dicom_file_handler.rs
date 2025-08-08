use crate::producer::KafkaProducer;
use common::DicomMessage;
use dicom_dictionary_std::tags;
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use snafu::{ResultExt, Whatever};
use tracing::info;

pub(crate) async fn process_dicom_file(
    kafka_producer: &KafkaProducer, // 添加这一行
    instance_buffer: &[u8],         //DICOM文件的字节数组或是二进制流
    out_dir: &str,                 //存储文件的根目录, 例如 :/opt/dicomStore/
    issue_patient_id: &String,      // 机构ID,或是医院ID, 用于区分多个医院.
    ts: &String,                    // 传输语法
    sop_instance_uid: &String,      //当前文件的SOP实例ID
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
        .trim_end_matches("\0").to_string();

    let study_uid = obj
        .element(tags::STUDY_INSTANCE_UID)
        .whatever_context("Missing StudyID")?
        .to_str()
        .whatever_context("could not retrieve STUDY_INSTANCE_UID")?
        .trim_end_matches("\0").to_string();

    let series_uid = obj
        .element(tags::SERIES_INSTANCE_UID)
        .whatever_context("Missing SeriesID")?
        .to_str()
        .whatever_context("could not retrieve SERIES_INSTANCE_UID")?
        .trim_end_matches("\0").to_string();

    info!(
        "Issur:{} ,PatientID: {}, StudyUID: {}, SeriesUID: {}",
        issue_patient_id, pat_id, study_uid, series_uid
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
                out_dir,
                issue_patient_id,
                pat_id ,
                study_uid,
                series_uid
            )
        }
        false => {
            format!(
                "{}/{}/{}/{}/{}",
                out_dir,
                issue_patient_id,
                pat_id ,
                study_uid,
                series_uid
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
        .whatever_context("could not save DICOM object to file")?;
    info!("Stored {}, {}", ts, sop_instance_uid);

    let dicom_message = DicomMessage {
        tenant: issue_patient_id.to_string(),
        transfer_syntax: ts.to_string(),
        sop_instance_uid: sop_instance_uid.to_string(),
        study_instance_uid: study_uid.to_string(),
        series_instance_uid: series_uid.to_string(),
        patient_id: pat_id.to_string(),
        file_path: file_path.to_string(),
        file_size: instance_buffer.len() as u64,
    };

    // 1. 发送到存储队列
    kafka_producer
        .send_message(
            "storage_queue",
            &dicom_message.sop_instance_uid,
            &dicom_message,
        )
        .await
        .unwrap();

    // 2. 发送到索引队列
    kafka_producer
        .send_message(
            "index_queue",
            &dicom_message.sop_instance_uid,
            &dicom_message,
        )
        .await
        .unwrap();

    Ok(())
}
