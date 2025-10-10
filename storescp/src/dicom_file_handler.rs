use common::database_entities::DicomObjectMeta;
use common::dicom_utils::get_tag_value;
use common::utils::get_logger;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::{ResultExt, Whatever};
use dicom_encoding::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use slog::{debug, error, info};
use slog::o;
use std::path::PathBuf;
use common::message_sender_kafka::KafkaMessagePublisher;

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
        },
        _ => return false,
    };

    day <= days_in_month
}

pub(crate) async fn process_dicom_file(
    instance_buffer: &[u8],    //DICOM文件的字节数组或是二进制流
    out_dir: &str,             //存储文件的根目录, 例如 :/opt/dicomStore/
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

    let sop_uid = obj
        .element(tags::SOP_INSTANCE_UID)
        .whatever_context("Missing SOP_INSTANCE_UID")?
        .to_str()
        .whatever_context("could not retrieve SOP_INSTANCE_UID")?
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
    //TODO: 校验StudyDate格式是否正确. YYYYMMDD 格式
    // 校验 StudyDate 格式
    validate_study_date_format(&study_date)
        .whatever_context("Invalid StudyDate format")?;
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

    debug!(logger, "file path: {}", dir_path);

    let ok = std::fs::exists(&dir_path);
    match ok {
        Ok(false) => {
            std::fs::create_dir_all(&dir_path).unwrap_or_else(|e| {
                // 创建目录失败, 记录错误日志并 panic. 因为无法创建目录, 后续写文件操作也无法进行.程序立即退出.
                //
                // 原则上这是一个不应该出现的错误. 因为在程序启动的时候,已经检查了目录是否具有读写权限.
                //
                error!(logger, "create directory failed: {}, error: {}", dir_path, e);
                panic!("create directory failed: {}", dir_path);
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
    // // 从新从磁盘读取DICOM文件, 确保文件已经完全写入磁盘.
    // let dicom_obj = dicom_object::OpenFileOptions::new()
    //     .charset_override(CharacterSetOverride::AnyVr)
    //     .read_until(tags::PIXEL_DATA)
    //     .open_file(&file_path)
    //     .whatever_context("open dicom file failed")?;

    // 修复后：
    let saved_path = PathBuf::from(file_path); // 此时可以安全转移所有权

    lst.push(DicomObjectMeta {
        tenant_id: tenant_id.to_string(),
        patient_id: pat_id.to_string(),
        study_uid: study_uid.to_string(),
        series_uid: series_uid.to_string(),
        sop_uid: sop_uid.to_string(),
        file_path: String::from(saved_path.to_str().unwrap()),
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
    storage_producer: &KafkaMessagePublisher,
    change_producer: &KafkaMessagePublisher,
    queue_topic_main: &str,
    queue_topic_change: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("storescp"=>"classify_and_publish_dicom_messages"));
    // 将 dicom_message_lists 按 transfer_syntax_uid 是否被 SUPPORTED_TRANSFER_SYNTAXES 支持分成两类
    let (supported_messages, unsupported_messages): (Vec<_>, Vec<_>) =
        dicom_message_lists.iter().partition(|meta| {
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
