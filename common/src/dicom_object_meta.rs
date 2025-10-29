use crate::dicom_utils;
use crate::string_ext::{BoundedString, DicomDateString, SopUidString, UidHashString, UuidString};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransferStatus {
    NoNeedTransfer,
    Success,
    Failed,
}

/// DicomStoreMeta 用于DICOM-CStoreSCP服务记录收图日志.
/// 包含了所有必要的元数据字段.每一个DicomStoreMeta实例标识接收一个DICOM文件.并成功写入磁盘.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStoreMeta {
    #[serde(rename = "trace_id")]
    pub trace_id: UuidString,
    #[serde(rename = "worker_node_id")]
    pub worker_node_id: BoundedString<64>,
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: SopUidString,
    #[serde(rename = "series_uid")]
    pub series_uid: SopUidString,
    #[serde(rename = "sop_uid")]
    pub sop_uid: SopUidString,
    #[serde(rename = "file_size")]
    pub file_size: u32,
    #[serde(rename = "file_path")]
    pub file_path: BoundedString<512>,
    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: SopUidString,
    #[serde(rename = "number_of_frames")]
    pub number_of_frames: i32,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: UidHashString,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: UidHashString,
    #[serde(rename = "accession_number")]
    pub accession_number: BoundedString<16>,
    #[serde(rename = "target_ts")]
    pub target_ts: SopUidString,
    #[serde(rename = "study_date")]
    pub study_date: NaiveDate,
    #[serde(rename = "transfer_status")]
    pub transfer_status: TransferStatus,
    #[serde(rename = "source_ip")]
    pub source_ip: BoundedString<24>,
    #[serde(rename = "source_ae")]
    pub source_ae: BoundedString<64>,
}
// 为 DicomObjectMeta 实现 Hash trait 以便可以在 HashSet 中使用
impl Hash for DicomStoreMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tenant_id.hash(state);
        self.patient_id.hash(state);
        self.study_uid.hash(state);
        self.series_uid.hash(state);
        self.sop_uid.hash(state);
    }
}
impl PartialEq for DicomStoreMeta {
    fn eq(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.patient_id == other.patient_id
            && self.study_uid == other.study_uid
            && self.series_uid == other.series_uid
            && self.sop_uid == other.sop_uid
    }
}
impl Eq for DicomStoreMeta {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DicomParseError {
    MissingRequiredField(String),
    InvalidTimeFormat(String),
    InvalidDateFormat(String),
    InvalidFormat(String),

    TransferSyntaxUidIsEmpty(String),
    SopClassUidIsEmpty(String),
    ConversionError(String),
    // 可以根据需要添加其他错误类型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStateMeta {
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: SopUidString,
    #[serde(rename = "series_uid")]
    pub series_uid: SopUidString,
    #[serde(rename = "study_uid_hash")]

    pub study_uid_hash: UidHashString,
    #[serde(rename = "series_uid_hash")]

    pub series_uid_hash: UidHashString,
    #[serde(rename = "study_date_origin")]
    pub study_date_origin: DicomDateString,

    #[serde(rename = "patient_name")]
    pub patient_name: Option<BoundedString<64>>,
    #[serde(rename = "patient_sex")]
    pub patient_sex: Option<BoundedString<1>>,
    #[serde(rename = "patient_birth_date")]
    pub patient_birth_date: Option<NaiveDate>,
    #[serde(rename = "patient_birth_time")]
    pub patient_birth_time: Option<NaiveTime>,
    #[serde(rename = "patient_age")]
    pub patient_age: Option<BoundedString<16>>,
    #[serde(rename = "patient_size")]
    pub patient_size: Option<f64>,
    #[serde(rename = "patient_weight")]
    pub patient_weight: Option<f64>,



    #[serde(rename = "study_date")]
    pub study_date: NaiveDate,
    #[serde(rename = "study_time")]
    pub study_time: Option<NaiveTime>,
    #[serde(rename = "accession_number")]
    pub accession_number: BoundedString<16>,
    #[serde(rename = "study_id")]
    pub study_id: Option<BoundedString<16>>,
    #[serde(rename = "study_description")]
    pub study_description: Option<BoundedString<64>>,

    #[serde(rename = "modality")]
    pub modality: Option<BoundedString<16>>,
    #[serde(rename = "series_number")]
    pub series_number: Option<i32>,
    #[serde(rename = "series_date")]
    pub series_date: Option<NaiveDate>,
    #[serde(rename = "series_time")]
    pub series_time: Option<NaiveTime>,
    #[serde(rename = "series_description")]
    pub series_description: Option<BoundedString<256>>,
    #[serde(rename = "body_part_examined")]
    pub body_part_examined: Option<BoundedString<64>>,
    #[serde(rename = "protocol_name")]
    pub protocol_name: Option<BoundedString<64>>,
    #[serde(rename = "series_related_instances")]
    pub series_related_instances: Option<i32>,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "updated_time")]
    pub updated_time: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomImageMeta {
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,

    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,

    #[serde(rename = "study_uid")]
    pub study_uid: SopUidString,

    #[serde(rename = "series_uid")]
    pub series_uid: SopUidString,

    #[serde(rename = "sop_uid")]
    pub sop_uid: SopUidString,

    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: UidHashString,

    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: UidHashString,

    #[serde(rename = "instance_number")]
    pub instance_number: Option<i32>,

    #[serde(rename = "content_date")]
    pub content_date: Option<DicomDateString>,

    #[serde(rename = "content_time")]
    pub content_time: Option<NaiveTime>,

    #[serde(rename = "image_type")]
    pub image_type: Option<BoundedString<128>>,

    #[serde(rename = "image_orientation_patient")]
    pub image_orientation_patient: Option<BoundedString<128>>,

    #[serde(rename = "image_position_patient")]
    pub image_position_patient: Option<BoundedString<64>>,

    #[serde(rename = "slice_thickness")]
    pub slice_thickness: Option<f64>,

    #[serde(rename = "spacing_between_slices")]
    pub spacing_between_slices: Option<f64>,

    #[serde(rename = "slice_location")]
    pub slice_location: Option<f64>,

    #[serde(rename = "samples_per_pixel")]
    pub samples_per_pixel: Option<i32>,

    #[serde(rename = "photometric_interpretation")]
    pub photometric_interpretation: Option<BoundedString<32>>,

    #[serde(rename = "width")]
    pub width: Option<i32>,

    #[serde(rename = "columns")]
    pub columns: Option<i32>,

    #[serde(rename = "bits_allocated")]
    pub bits_allocated: Option<i32>,

    #[serde(rename = "bits_stored")]
    pub bits_stored: Option<i32>,

    #[serde(rename = "high_bit")]
    pub high_bit: Option<i32>,

    #[serde(rename = "pixel_representation")]
    pub pixel_representation: Option<i32>,

    #[serde(rename = "rescale_intercept")]
    pub rescale_intercept: Option<f64>,

    #[serde(rename = "rescale_slope")]
    pub rescale_slope: Option<f64>,

    #[serde(rename = "rescale_type")]
    pub rescale_type: Option<BoundedString<64>>,

    #[serde(rename = "window_center")]
    pub window_center: Option<BoundedString<64>>,

    #[serde(rename = "window_width")]
    pub window_width: Option<BoundedString<64>>,

    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: SopUidString,

    #[serde(rename = "pixel_data_location")]
    pub pixel_data_location: Option<BoundedString<512>>,

    #[serde(rename = "thumbnail_location")]
    pub thumbnail_location: Option<BoundedString<512>>,

    #[serde(rename = "sop_class_uid")]
    pub sop_class_uid: SopUidString,

    #[serde(rename = "image_status")]
    pub image_status: Option<BoundedString<32>>,

    #[serde(rename = "space_size")]
    pub space_size: Option<u32>,

    #[serde(rename = "created_time")]
    pub created_time: Option<NaiveDateTime>,

    #[serde(rename = "updated_time")]
    pub updated_time: Option<NaiveDateTime>,
}

impl DicomStateMeta {
    /// 基于 tenant_id, patient_id, study_uid, series_uid 创建唯一标识符
    pub fn unique_key(&self) -> (String, String, String, String) {
        (
            self.tenant_id.as_str().to_string(),
            self.patient_id.as_str().to_string(),
            self.study_uid.as_str().to_string(),
            self.series_uid.as_str().to_string(),
        )
    }
}
struct DicomCommonMeta {
    patient_id: String,
    study_uid: String,
    series_uid: String,
    sop_uid: String,
    study_date: NaiveDate,
    study_date_str: String,
}

impl DicomCommonMeta {
    fn extract_from_dicom(dicom_obj: &InMemDicomObject) -> Result<Self, DicomParseError> {
        // 提取 patient_id
        let patient_id_str = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_ID)
            .filter(|v| !v.is_empty() && v.len() <= 64)
            .ok_or_else(|| DicomParseError::MissingRequiredField("Patient ID".to_string()))?;

        // 提取 study_uid
        let study_uid = dicom_utils::get_text_value(dicom_obj, tags::STUDY_INSTANCE_UID)
            .filter(|v| !v.is_empty() && v.len() <= 64)
            .ok_or_else(|| {
                DicomParseError::MissingRequiredField("Study Instance UID".to_string())
            })?;

        // 提取 series_uid
        let series_uid = dicom_utils::get_text_value(dicom_obj, tags::SERIES_INSTANCE_UID)
            .filter(|v| !v.is_empty() && v.len() <= 64)
            .ok_or_else(|| {
                DicomParseError::MissingRequiredField("Series Instance UID".to_string())
            })?;

        // 提取 sop_uid (仅对 DicomImageMeta 需要)
        let sop_uid = dicom_utils::get_text_value(dicom_obj, tags::SOP_INSTANCE_UID)
            .filter(|v| !v.is_empty() && v.len() <= 64)
            .ok_or_else(|| DicomParseError::MissingRequiredField("SOP Instance UID".to_string()))?;

        // 提取 study_date_str
        let study_date_str =
            dicom_utils::get_text_value(dicom_obj, tags::STUDY_DATE).ok_or_else(|| {
                DicomParseError::MissingRequiredField("Study Date text value".to_string())
            })?;

        // 验证 study_date_str 格式
        if study_date_str.len() != 8 || !study_date_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(DicomParseError::InvalidDateFormat(format!(
                "Study Date must be in YYYYMMDD format, got: {}",
                study_date_str
            )));
        }

        Ok(DicomCommonMeta {
            patient_id: patient_id_str,
            study_uid,
            series_uid,
            sop_uid,
            study_date: NaiveDate::parse_from_str(&study_date_str, "%Y%m%d").unwrap(),
            study_date_str,
        })
    }
}

/// 对 Vec<DicomStateMeta> 进行去重处理

pub fn make_image_info(
    tenant_id: &str,
    dicom_obj: &InMemDicomObject,
    fsize: Option<u32>,
) -> Result<DicomImageMeta, DicomParseError> {
    // 使用公共提取器获取基本信息
    let common_meta = DicomCommonMeta::extract_from_dicom(dicom_obj)?;
    // 图像相关信息
    let instance_number = dicom_utils::get_int_value(dicom_obj, tags::INSTANCE_NUMBER);

    let content_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::CONTENT_DATE)
        .map(|date| {
            let date_str = date.format("%Y%m%d").to_string();
            DicomDateString::try_from(date_str)
        })
        .transpose()
        .map_err(|_| {
            DicomParseError::InvalidDateFormat("Failed to convert content date".to_string())
        })?;

    let content_time = dicom_utils::get_text_value(dicom_obj, tags::CONTENT_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| parse_dicom_time(v.as_str()))
        .transpose()
        .map_err(|_| {
            DicomParseError::InvalidTimeFormat("Failed to convert content_time".to_string())
        })?;

    let image_type = dicom_utils::get_text_value(dicom_obj, tags::IMAGE_TYPE)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<128>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert image type".to_string())
        })?;

    let image_orientation_patient =
        dicom_utils::get_text_value(dicom_obj, tags::IMAGE_ORIENTATION_PATIENT)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<128>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert image orientation patient".to_string(),
                )
            })?;
    let image_position_patient =
        dicom_utils::get_text_value(dicom_obj, tags::IMAGE_POSITION_PATIENT)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<64>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert image position patient".to_string(),
                )
            })?;

    let slice_thickness = dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_THICKNESS);
    let spacing_between_slices =
        dicom_utils::get_decimal_value(dicom_obj, tags::SPACING_BETWEEN_SLICES);
    let slice_location = dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_LOCATION);

    let samples_per_pixel = dicom_utils::get_int_value(dicom_obj, tags::SAMPLES_PER_PIXEL);
    let photometric_interpretation =
        dicom_utils::get_text_value(dicom_obj, tags::PHOTOMETRIC_INTERPRETATION)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<32>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert photometric interpretation".to_string(),
                )
            })?;

    let width = dicom_utils::get_int_value(dicom_obj, tags::ROWS);
    let columns = dicom_utils::get_int_value(dicom_obj, tags::COLUMNS);
    let bits_allocated = dicom_utils::get_int_value(dicom_obj, tags::BITS_ALLOCATED);
    let bits_stored = dicom_utils::get_int_value(dicom_obj, tags::BITS_STORED);
    let high_bit = dicom_utils::get_int_value(dicom_obj, tags::HIGH_BIT);
    let pixel_representation = dicom_utils::get_int_value(dicom_obj, tags::PIXEL_REPRESENTATION);

    let rescale_intercept = dicom_utils::get_decimal_value(dicom_obj, tags::RESCALE_INTERCEPT);
    let rescale_slope = dicom_utils::get_decimal_value(dicom_obj, tags::RESCALE_SLOPE);
    let rescale_type = dicom_utils::get_text_value(dicom_obj, tags::RESCALE_TYPE)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert rescale type".to_string())
        })?;

    let window_center = dicom_utils::get_text_value(dicom_obj, tags::WINDOW_CENTER)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert window center".to_string())
        })?;
    let window_width = dicom_utils::get_text_value(dicom_obj, tags::WINDOW_WIDTH)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert window width".to_string())
        })?;
    let transfer_syntax_uid = dicom_utils::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "1.2.840.10008.1.2".to_string());

    let sop_class_uid = dicom_utils::get_text_value(dicom_obj, tags::SOP_CLASS_UID)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| DicomParseError::MissingRequiredField("SOP Class UID".to_string()))?;

    let image_status = Some(
        BoundedString::<32>::try_from("ACTIVE".to_string()).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert image status".to_string())
        })?,
    );

    // 计算哈希值
    let study_uid_hash = UidHashString::make_from_db(&common_meta.study_uid.as_str());
    let series_uid_hash = UidHashString::make_from_db(&common_meta.series_uid.as_str());

    // 时间戳
    let now =  chrono::Local::now().naive_local();

    Ok(DicomImageMeta {
        tenant_id: BoundedString::<64>::try_from(tenant_id.to_string()).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert tenant ID".to_string())
        })?,
        patient_id: BoundedString::<64>::try_from(common_meta.patient_id).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient ID".to_string())
        })?,
        study_uid: SopUidString::try_from(&common_meta.study_uid).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert study UID".to_string())
        })?,
        series_uid: SopUidString::try_from(&common_meta.series_uid).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert series UID".to_string())
        })?,
        sop_uid: SopUidString::try_from(&common_meta.sop_uid).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert SOP UID".to_string())
        })?,
        study_uid_hash,
        series_uid_hash,

        instance_number,

        content_date,
        content_time,

        image_type,
        image_orientation_patient,
        image_position_patient,
        slice_thickness,
        spacing_between_slices,
        slice_location,
        samples_per_pixel,
        photometric_interpretation,
        width,
        columns,
        bits_allocated,
        bits_stored,
        high_bit,
        pixel_representation,
        rescale_intercept,
        rescale_slope,
        rescale_type,
        window_center,
        window_width,

        transfer_syntax_uid: SopUidString::try_from(transfer_syntax_uid).unwrap(),
        pixel_data_location: None,
        thumbnail_location: None,
        sop_class_uid: SopUidString::try_from(sop_class_uid).map_err(|_| {
            DicomParseError::SopClassUidIsEmpty("SOP Class UID is empty".to_string())
        })?,
        image_status,
        space_size: fsize,
        created_time: Some(now),
        updated_time: Some(now),
    })
}

fn make_crc32(tenante_id: &str, study_uid: Option<&str>) -> u32 {
    let mut data = vec![0u8; 128];
    data[..tenante_id.len()].copy_from_slice(tenante_id.as_bytes());
    if let Some(study_uid) = study_uid {
        data[tenante_id.len()..tenante_id.len() + study_uid.len()]
            .copy_from_slice(study_uid.as_bytes());
    }
    const_crc32::crc32(&data)
}
/// 解析DICOM时间字符串，支持多种格式：
/// - %H%M%S.%f (带毫秒)
/// - %H%M%S. (带点但无毫秒)
/// - %H%M%S (不带毫秒)
fn parse_dicom_time(time_str: &str) -> Result<NaiveTime, chrono::ParseError> {
    // 尝试解析带毫秒的格式 (%H%M%S.%f)
    NaiveTime::parse_from_str(time_str, "%H%M%S.%f")
        .or_else(|_| {
            // 尝试解析带点但无毫秒的格式 (%H%M%S.)
            NaiveTime::parse_from_str(time_str, "%H%M%S.")
        })
        .or_else(|_| {
            // 尝试解析不带毫秒的格式 (%H%M%S)
            NaiveTime::parse_from_str(time_str, "%H%M%S")
        })
}
pub fn make_state_info(
    tenant_id: &str,

    dicom_obj: &InMemDicomObject,
    msg_study_uid: Option<&str>,
) -> Result<DicomStateMeta, DicomParseError> {
    // 必填字段验证 - 确保不为空
    // 必填字段验证 - 确保不为空且长度不超过64
    // 使用公共提取器获取基本信息
    let common_meta = DicomCommonMeta::extract_from_dicom(dicom_obj)?;

    let acc_num = dicom_utils::get_text_value(dicom_obj, tags::ACCESSION_NUMBER)
        .filter(|v| !v.is_empty() && v.len() <= 16)
        .unwrap_or_else(|| format!("X32CRC{}", make_crc32(tenant_id, msg_study_uid))); // 当为空时设置默认值"X12333"
    let modality = dicom_utils::get_text_value(dicom_obj, tags::MODALITY)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| DicomParseError::ConversionError("Failed to convert modality".to_string()))?;

    // 患者相关信息
    let patient_name = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_NAME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient name".to_string())
        })?;

    let patient_sex = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_SEX)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<1>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient sex".to_string())
        })?;

    let patient_birth_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::PATIENT_BIRTH_DATE);

    let patient_birth_time = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_BIRTH_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| parse_dicom_time(v.as_str()))
        .transpose()
        .map_err(|_| {
            DicomParseError::InvalidTimeFormat("Failed to convert patient birth time".to_string())
        })?;

    // 患者其他信息
    let patient_age = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_AGE)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient age".to_string())
        })?;

    let patient_size = dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_SIZE);
    let patient_weight = dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_WEIGHT);


    let study_date = common_meta.study_date;
    // 检查相关信息
    let study_time = dicom_utils::get_text_value(dicom_obj, tags::STUDY_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| parse_dicom_time(v.as_str()))
        .transpose()
        .map_err(|_| {
            DicomParseError::InvalidTimeFormat("Failed to convert study time".to_string())
        })?;

    let study_id = dicom_utils::get_text_value(dicom_obj, tags::STUDY_ID)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| DicomParseError::ConversionError("Failed to convert study ID".to_string()))?;

    let study_description = dicom_utils::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert study description".to_string())
        })?;

    // let referring_physician_name =
    //     dicom_utils::get_text_value(dicom_obj, tags::REFERRING_PHYSICIAN_NAME)
    //         .filter(|v| !v.is_empty())
    //         .map(|v| BoundedString::<64>::try_from(v))
    //         .transpose()
    //         .map_err(|_| {
    //             DicomParseError::ConversionError(
    //                 "Failed to convert referring physician name".to_string(),
    //             )
    //         })?;
    //
    // let admission_id = dicom_utils::get_text_value(dicom_obj, tags::ADMISSION_ID)
    //     .filter(|v| !v.is_empty())
    //     .map(|v| BoundedString::<64>::try_from(v))
    //     .transpose()
    //     .map_err(|_| {
    //         DicomParseError::ConversionError("Failed to convert admission ID".to_string())
    //     })?;
    //
    // let performing_physician_name =
    //     dicom_utils::get_text_value(dicom_obj, tags::PERFORMING_PHYSICIAN_NAME)
    //         .filter(|v| !v.is_empty())
    //         .map(|v| BoundedString::<64>::try_from(v))
    //         .transpose()
    //         .map_err(|_| {
    //             DicomParseError::ConversionError(
    //                 "Failed to convert performing physician name".to_string(),
    //             )
    //         })?;

    // 序列相关信息

    let series_number = dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER);
    let series_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE);

    let series_time = dicom_utils::get_text_value(dicom_obj, tags::SERIES_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| parse_dicom_time(v.as_str()))
        .transpose()
        .map_err(|_| {
            DicomParseError::InvalidTimeFormat("Failed to convert series time".to_string())
        })?;

    let series_description = dicom_utils::get_text_value(dicom_obj, tags::SERIES_DESCRIPTION)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<256>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert series description".to_string())
        })?;

    let body_part_examined = dicom_utils::get_text_value(dicom_obj, tags::BODY_PART_EXAMINED)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert body part examined".to_string())
        })?;

    let protocol_name = dicom_utils::get_text_value(dicom_obj, tags::PROTOCOL_NAME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert protocol name".to_string())
        })?;

    // let operators_name = dicom_utils::get_text_value(dicom_obj, tags::OPERATORS_NAME)
    //     .filter(|v| !v.is_empty())
    //     .map(|v| BoundedString::<64>::try_from(v))
    //     .transpose()
    //     .map_err(|_| {
    //         DicomParseError::ConversionError("Failed to convert operators name".to_string())
    //     })?;

    // let manufacturer = dicom_utils::get_text_value(dicom_obj, tags::MANUFACTURER)
    //     .filter(|v| !v.is_empty())
    //     .map(|v| BoundedString::<64>::try_from(v))
    //     .transpose()
    //     .map_err(|_| {
    //         DicomParseError::ConversionError("Failed to convert manufacturer".to_string())
    //     })?;
    //
    // let institution_name = dicom_utils::get_text_value(dicom_obj, tags::INSTITUTION_NAME)
    //     .filter(|v| !v.is_empty())
    //     .map(|v| BoundedString::<64>::try_from(v))
    //     .transpose()
    //     .map_err(|_| {
    //         DicomParseError::ConversionError("Failed to convert institution name".to_string())
    //     })?;
    // let device_serial_number = dicom_utils::get_text_value(dicom_obj, tags::DEVICE_SERIAL_NUMBER)
    //     .filter(|v| !v.is_empty())
    //     .map(|v| BoundedString::<64>::try_from(v))
    //     .transpose()
    //     .map_err(|_| {
    //         DicomParseError::ConversionError("Failed to convert device serial number".to_string())
    //     })?;
    //
    // let software_versions = dicom_utils::get_text_value(dicom_obj, tags::SOFTWARE_VERSIONS)
    //     .filter(|v| !v.is_empty())
    //     .map(|v| BoundedString::<64>::try_from(v))
    //     .transpose()
    //     .map_err(|_| {
    //         DicomParseError::ConversionError("Failed to convert software versions".to_string())
    //     })?;
    let series_related_instances =
        dicom_utils::get_int_value(dicom_obj, tags::NUMBER_OF_SERIES_RELATED_INSTANCES);

    // 计算哈希值
    let study_uid_hash = UidHashString::make_from_db(&common_meta.study_uid.as_str());
    let series_uid_hash = UidHashString::make_from_db(&common_meta.series_uid.as_str());

    // 时间戳
    let now = chrono::Local::now().naive_local();
    let study_date_origin = DicomDateString::try_from(&common_meta.study_date_str).unwrap();

    let tenant_id = BoundedString::<64>::try_from(tenant_id.to_string()).unwrap();
    let patient_id = BoundedString::<64>::try_from(&common_meta.patient_id).unwrap();
    let study_uid = SopUidString::try_from(&common_meta.study_uid).unwrap();
    let series_uid = SopUidString::try_from(&common_meta.series_uid).unwrap();
    let accession_number = BoundedString::<16>::try_from(acc_num).unwrap();

    Ok(DicomStateMeta {
        tenant_id,
        patient_id,
        study_uid,
        series_uid,
        study_uid_hash,
        series_uid_hash,
        study_date_origin,
        // 患者信息
        patient_name,
        patient_sex,
        patient_birth_date,
        patient_birth_time,
        patient_age,
        patient_size,
        patient_weight,

        // 检查信息
        study_date,
        study_time,
        accession_number,
        study_id,
        study_description,

        // 序列信息
        modality,
        series_number,
        series_date,
        series_time,
        series_description,
        body_part_examined,
        protocol_name,

        series_related_instances,
        // 时间戳
        created_time: now,
        updated_time: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use dicom_object::collector::CharacterSetOverride;
    use rstest::rstest;
    use std::fs;
    use std::path::Path;

    // 递归收集目录及其子目录中的所有.dcm文件
    fn collect_dicom_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 递归遍历子目录
                    collect_dicom_files(&path, files);
                } else if path
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("dcm"))
                {
                    // 添加.dcm文件到列表
                    files.push(path);
                }
            }
        }
    }
    #[rstest]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/107")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/108")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/109")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/110")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/111")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/112")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/113")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/114")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/115")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/116")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/117")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/118")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/119")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/120")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/121")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/122")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/123")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/124")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/125")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/126")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/127")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/128")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/129")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/130")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/131")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/132")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/133")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/134")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/135")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/136")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/137")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/138")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/139")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/140")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/141")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/142")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/143")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/144")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/145")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/146")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/147")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/148")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/149")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/150")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/151")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/152")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/153")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/154")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/4.90")]
    #[case("/home/dhz/jpdata/CDSS/DicomTest/89269")]

    fn test_make_state_info_with_dicom_files(#[case] dicom_dir: &str) {
        // let dicom_dir = "/media/dhz/DCP/DicomTestDataSet/dcmFiles/103";

        // 检查目录是否存在
        if !Path::new(dicom_dir).exists() {
            println!("DICOM test directory not found: {}", dicom_dir);
            return;
        }

        // 递归遍历目录及其子目录中的所有.dcm文件
        let mut dicom_files = Vec::new();

        // 使用递归函数收集所有.dcm文件
        collect_dicom_files(Path::new(dicom_dir), &mut dicom_files);

        println!("Found {} DICOM files", dicom_files.len());

        for (index, path) in dicom_files.iter().enumerate() {
            println!(
                "Processing file {}/{}: {:?}",
                index + 1,
                dicom_files.len(),
                path
            );

            // 尝试打开DICOM文件
            match dicom_object::OpenFileOptions::new()
                .charset_override(CharacterSetOverride::AnyVr)
                .read_until(tags::PIXEL_DATA)
                .open_file(path)
            {
                Ok(dicom_obj) => {
                    // 尝试解析DICOM对象
                    let result = make_state_info("1234567890", &dicom_obj, None);

                    match result {
                        Ok(state_meta) => {
                            // 验证必填字段
                            assert!(
                                !state_meta.patient_id.as_str().is_empty(),
                                "Patient ID should not be empty in file: {:?}",
                                path
                            );
                            assert!(
                                !state_meta.study_uid.as_str().is_empty(),
                                "Study UID should not be empty in file: {:?}",
                                path
                            );
                            assert!(
                                !state_meta.series_uid.as_str().is_empty(),
                                "Series UID should not be empty in file: {:?}",
                                path
                            );
                            assert!(
                                !state_meta.modality.unwrap().as_str().is_empty(),
                                "Modality should not be empty in file: {:?}",
                                path
                            );

                            println!("Successfully parsed file: {:?}", path);
                        }
                        Err(e) => {
                            eprintln!("Error parsing file {:?}: {:?}", path, e);
                            panic!("Failed to parse DICOM file {:?}: {:?}", path, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error opening file {:?}: {:?}", path, e);
                    // std::fs::remove_file(path).expect("Failed to delete file");
                }
            }
        }

        println!("All DICOM files processed successfully");
    }

    // #[rstest]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/107")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/108")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/109")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/110")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/111")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/112")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/113")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/114")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/115")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/116")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/117")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/118")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/119")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/120")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/121")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/122")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/123")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/124")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/125")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/126")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/127")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/128")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/129")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/130")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/131")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/132")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/133")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/134")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/135")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/136")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/137")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/138")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/139")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/140")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/141")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/142")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/143")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/144")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/145")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/146")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/147")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/148")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/149")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/150")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/151")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/152")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/153")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/dcmFiles/154")]
    // #[case("/media/dhz/DCP/DicomTestDataSet/4.90")]

    // fn test_make_image_info_with_dicom_files(#[case] dicom_dir: &str) {
    //     // let dicom_dir = "/media/dhz/DCP/DicomTestDataSet/dcmFiles/103";
    //
    //     // 检查目录是否存在
    //     if !Path::new(dicom_dir).exists() {
    //         println!("DICOM test directory not found: {}", dicom_dir);
    //         return;
    //     }
    //
    //     // 递归遍历目录及其子目录中的所有.dcm文件
    //     let mut dicom_files = Vec::new();
    //
    //     // 使用递归函数收集所有.dcm文件
    //     collect_dicom_files(Path::new(dicom_dir), &mut dicom_files);
    //
    //     println!("Found {} DICOM files", dicom_files.len());
    //
    //     for (index, path) in dicom_files.iter().enumerate() {
    //         println!(
    //             "Processing file {}/{}: {:?}",
    //             index + 1,
    //             dicom_files.len(),
    //             path
    //         );
    //
    //         // 尝试打开DICOM文件
    //         match dicom_object::OpenFileOptions::new()
    //             .charset_override(CharacterSetOverride::AnyVr)
    //             .read_until(tags::PIXEL_DATA)
    //             .open_file(path)
    //         {
    //             Ok(dicom_obj) => {
    //                 // 尝试解析DICOM对象
    //                 let result = make_image_info("1234567890", &dicom_obj, None);
    //
    //                 match result {
    //                     Ok(state_meta) => {
    //                         // 验证必填字段
    //                         assert!(
    //                             !state_meta.patient_id.as_str().is_empty(),
    //                             "Patient ID should not be empty in file: {:?}",
    //                             path
    //                         );
    //                         assert!(
    //                             !state_meta.study_uid.as_str().is_empty(),
    //                             "Study UID should not be empty in file: {:?}",
    //                             path
    //                         );
    //                         assert!(
    //                             !state_meta.series_uid.as_str().is_empty(),
    //                             "Series UID should not be empty in file: {:?}",
    //                             path
    //                         );
    //                         assert!(
    //                             !state_meta.sop_uid.as_str().is_empty(),
    //                             "SOP UID should not be empty in file: {:?}",
    //                             path
    //                         );
    //
    //                         println!("Successfully parsed file: {:?}", path);
    //                     }
    //                     Err(e) => {
    //                         eprintln!("Error parsing file {:?}: {:?}", path, e);
    //                         panic!("Failed to parse DICOM file {:?}: {:?}", path, e);
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 eprintln!("Error opening file {:?}: {:?}", path, e);
    //                 // std::fs::remove_file(path).expect("Failed to delete file");
    //             }
    //         }
    //     }
    //
    //     println!("All DICOM files processed successfully");
    // }

    // #[test]
    // fn test_make_state_info_with_sample_files() {
    //     let dicom_dir = "/media/dhz/DCP/DicomTestDataSet";
    //
    //     // 检查目录是否存在
    //     if !Path::new(dicom_dir).exists() {
    //         println!("DICOM test directory not found: {}", dicom_dir);
    //         return;
    //     }
    //
    //     // 收集所有.dcm文件
    //     let mut dicom_files = Vec::new();
    //     collect_dicom_files(Path::new(dicom_dir), &mut dicom_files);
    //
    //     // 只测试前5个文件以避免测试时间过长
    //     let max_files = 5.min(dicom_files.len());
    //     let sample_files = &dicom_files[..max_files];
    //
    //     for (index, path) in sample_files.iter().enumerate() {
    //         println!("Processing file {}/{}: {:?}", index + 1, sample_files.len(), path);
    //
    //         // 尝试打开DICOM文件
    //         match open_file(path) {
    //             Ok(dicom_obj) => {
    //                 // 测试make_state_info函数
    //                 let result = make_state_info("1234567890", &dicom_obj);
    //
    //                 match result {
    //                     Ok(state_meta) => {
    //                         // 验证关键字段长度限制
    //                         assert!(
    //                             state_meta.patient_id.as_str().len() <= 64,
    //                             "Patient ID exceeds 64 characters in file: {:?}",
    //                             path
    //                         );
    //                         assert!(
    //                             state_meta.study_uid.as_str().len() <= 64,
    //                             "Study UID exceeds 64 characters in file: {:?}",
    //                             path
    //                         );
    //                         assert!(
    //                             state_meta.series_uid.as_str().len() <= 64,
    //                             "Series UID exceeds 64 characters in file: {:?}",
    //                             path
    //                         );
    //
    //                         println!("Successfully parsed file: {:?}", path);
    //                     }
    //                     Err(e) => {
    //                         eprintln!("Error parsing file {:?}: {:?}", path, e);
    //                         panic!("Failed to parse DICOM file {:?}: {:?}", path, e);
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 eprintln!("Error opening file {:?}: {:?}", path, e);
    //                 panic!("Failed to open DICOM file {:?}: {:?}", path, e);
    //             }
    //         }
    //     }
    //
    //     if dicom_files.is_empty() {
    //         println!("No .dcm files found in directory: {}", dicom_dir);
    //     } else {
    //         println!("Successfully processed {} DICOM files ({} sampled)", dicom_files.len(), max_files);
    //     }
    // }
}
