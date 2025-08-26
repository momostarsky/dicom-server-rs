use common::database_entities::DicomObjectMeta;

use common::producer_factory::KafkaProducer;
use dicom_dictionary_std::tags;
use common::database_provider_base::DbProviderBase;
use dicom_encoding::snafu::{whatever, ResultExt, Whatever};
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use tracing::info;
use tracing::log::error;

pub(crate) async fn process_dicom_file(
    instance_buffer: &[u8],    //DICOM文件的字节数组或是二进制流
    out_dir: &str,             //存储文件的根目录, 例如 :/opt/dicomStore/
    issue_patient_id: &String, // 机构ID,或是医院ID, 用于区分多个医院.
    ts: &String,               // 传输语法
    sop_instance_uid: &String, //当前文件的SOP实例ID
    lst: &mut Vec<common::database_entities::DicomObjectMeta>,
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



    let pat = match DbProviderBase::extract_patient_entity(issue_patient_id, &file_obj) {
        Some(pat) => pat,
        None => {
            whatever!("extract patient entity failed");
        }
    };
    let study = match DbProviderBase::extract_study_entity(
        issue_patient_id,
        &file_obj,
        pat.patient_id.as_str(),
    ) {
        Some(study) => study,
        None => {
            whatever!("extract study entity failed");
        }
    };

    let series = match DbProviderBase::extract_series_entity(
        issue_patient_id,
        &file_obj,
        study_uid.as_str(),
    ) {
        Some(series) => series,
        None => {
            whatever!("extract series entity failed");
        }
    };
    let mut image = match DbProviderBase::extract_image_entity(
        issue_patient_id,
        &file_obj,
        study_uid.as_str(),
        series_uid.as_str(),
        pat.patient_id.as_str(),
    ) {
        Some(image) => image,
        None => {
            whatever!("extract image entity failed");
        }
    };

    let fs = std::fs::metadata(file_path.as_str());
    let file_size = match fs {
        Ok(fs) => fs.len() as u64,
        Err(_) => 0,
    };
    println!("image is {:?}", image);
    image.space_size = Option::from(file_size);
    image.transfer_syntax_uid = ts.to_string();
    lst.push(DicomObjectMeta {
        patient_info: pat,
        study_info: study,
        series_info: series,
        image_info: image.clone(),
        file_path: file_path.to_string(),
        file_size,
        tenant_id: issue_patient_id.to_string(),
        transfer_synatx_uid: ts.to_string(),
        number_of_frames: image.number_of_frames,
    });
    Ok(())
}

pub(crate) async fn publish_messages(
    main_kafka_producer: &KafkaProducer,
    multi_frames_kafka_producer: Option<&KafkaProducer>,
    chgts_kafka_producer: Option<&KafkaProducer>,
    dicom_message_lists: &Vec<DicomObjectMeta>,
) -> Result<(), Whatever> {
    if dicom_message_lists.is_empty() {
        return Ok(());
    }
    match main_kafka_producer
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

    if multi_frames_kafka_producer.is_none() && chgts_kafka_producer.is_none() {
        return Ok(());
    }

    match chgts_kafka_producer {
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

    match multi_frames_kafka_producer {
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
