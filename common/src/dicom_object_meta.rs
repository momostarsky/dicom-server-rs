use crate::string_ext::{BoundedString, DicomDateString, SopUidString, UuidString};
use crate::{dicom_utils, uid_hash};
use chrono::NaiveDateTime;
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;
use gdcm_conv::TransferSyntax::ImplicitVRLittleEndian;
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
    pub file_size: u64,
    #[serde(rename = "file_path")]
    pub file_path: BoundedString<512>,
    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: SopUidString,
    #[serde(rename = "number_of_frames")]
    pub number_of_frames: i32,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: u32,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: u64,
    #[serde(rename = "accession_number")]
    pub accession_number: BoundedString<64>,
    #[serde(rename = "target_ts")]
    pub target_ts: SopUidString,
    #[serde(rename = "study_date")]
    pub study_date: DicomDateString,
    #[serde(rename = "transfer_status")]
    pub transfer_status: TransferStatus,
    #[serde(rename = "source_ip")]
    pub source_ip: BoundedString<32>,
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
    InvalidFormat(String),
    ConversionError(String),
    TransferSyntaxUidIsEmpty(String),
    SopClassUidIsEmpty(String),
    // 可以根据需要添加其他错误类型
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStateMeta {
    pub tenant_id: BoundedString<64>,
    pub patient_id: BoundedString<64>,
    pub study_uid: SopUidString,
    pub series_uid: SopUidString,
    pub study_uid_hash: u64,
    pub series_uid_hash: u32,
    pub study_date_origin: DicomDateString,

    pub patient_name: Option<BoundedString<64>>,
    pub patient_sex: Option<BoundedString<1>>,
    pub patient_birth_date: Option<chrono::NaiveDate>,
    pub patient_birth_time: Option<BoundedString<16>>,
    pub patient_age: Option<BoundedString<16>>,
    pub patient_size: Option<f64>,
    pub patient_weight: Option<f64>,
    pub medical_alerts: Option<BoundedString<64>>,
    pub allergies: Option<BoundedString<64>>,
    pub pregnancy_status: Option<i32>,
    pub occupation: Option<BoundedString<64>>,
    pub additional_patient_history: Option<BoundedString<512>>,
    pub patient_comments: Option<BoundedString<512>>,

    pub study_date: chrono::NaiveDate,
    pub study_time: Option<BoundedString<16>>,
    pub accession_number: BoundedString<16>,
    pub study_id: Option<BoundedString<64>>,
    pub study_description: Option<BoundedString<64>>,
    pub referring_physician_name: Option<BoundedString<64>>,
    pub admission_id: Option<BoundedString<64>>,
    pub performing_physician_name: Option<BoundedString<64>>,
    pub procedure_code_sequence: Option<BoundedString<512>>,
    pub received_instances: Option<i32>,

    pub modality: String,
    pub series_number: Option<i32>,
    pub series_date: Option<chrono::NaiveDate>,
    pub series_time: Option<BoundedString<16>>,
    pub series_description: Option<BoundedString<256>>,
    pub body_part_examined: Option<BoundedString<64>>,
    pub protocol_name: Option<BoundedString<64>>,
    pub operators_name: Option<BoundedString<64>>,
    pub number_of_series_related_instances: Option<i32>,
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomImageMeta {
    pub tenant_id: BoundedString<64>,
    pub patient_id: BoundedString<64>,
    pub study_uid: SopUidString,
    pub series_uid: SopUidString,
    pub sop_uid: SopUidString,
    pub study_uid_hash: u64,
    pub series_uid_hash: u32,
    pub study_date_origin: DicomDateString,

    pub instance_number: Option<i32>,
    pub image_comments: Option<BoundedString<256>>,
    pub content_date: Option<DicomDateString>,
    pub content_time: Option<BoundedString<16>>,
    pub acquisition_date: Option<DicomDateString>,
    pub acquisition_time: Option<BoundedString<16>>,
    pub acquisition_date_time: Option<chrono::NaiveDateTime>,
    pub image_type: Option<BoundedString<256>>,
    pub image_orientation_patient: Option<String>,
    pub image_position_patient: Option<String>,
    pub slice_thickness: Option<f64>,
    pub spacing_between_slices: Option<f64>,
    pub slice_location: Option<f64>,
    pub samples_per_pixel: Option<i32>,
    pub photometric_interpretation: Option<BoundedString<32>>,
    pub width: Option<i32>,
    pub columns: Option<i32>,
    pub bits_allocated: Option<i32>,
    pub bits_stored: Option<i32>,
    pub high_bit: Option<i32>,
    pub pixel_representation: Option<i32>,
    pub rescale_intercept: Option<f64>,
    pub rescale_slope: Option<f64>,
    pub rescale_type: Option<String>,
    pub window_center: Option<String>,
    pub window_width: Option<String>,
    pub number_of_frames: i32,

    /*
    常见的应用场景
        图像重建算法：例如，区分使用了“滤波反投影 (Filtered Back Projection)”还是“迭代重建 (Iterative Reconstruction)”算法。
        后处理滤波：标识应用了哪些空间滤波器，如“锐化 (Edge Enhancement)”、“平滑 (Smoothing)”或“降噪 (Noise Reduction)”。
        特殊成像模式：用于标识特定的采集或处理模式，如“能谱成像处理”、“去金属伪影处理 (Metal Artifact Reduction)”等。
        数据校正：表示进行了哪些校正，如“散射校正”、“衰减校正”等。
    为什么需要它？
        互操作性：不同制造商的设备可能用不同的术语描述相似的处理。标准化的代码确保了信息在不同系统（如 PACS, RIS, 工作站）之间交换时的准确理解。
        自动化处理：下游系统（如 AI 分析工具、图像分析软件）可以根据这个代码来判断图像的处理状态，从而调整其分析算法或解释结果。例如，知道图像经过了强烈的锐化处理，可能会影响对边缘或纹理的分析。
        研究与质量保证：研究人员可以利用这些代码来筛选特定处理方式的图像集。质量保证流程可以检查预期的处理代码是否被正确应用。
     */
    // 设备处理描述  一个机器可读的、标准化的代码，确保不同厂商和系统之间对处理步骤的理解一致
    pub acquisition_device_processing_description: Option<BoundedString<256>>,
    // 设备处理代码  一个人类可读的文本描述（如 "Edge Enhancement", "Noise Reduction", "Filtered Back Projection"）
    pub acquisition_device_processing_code: Option<BoundedString<256>>,
    /*
    核心功能
    唯一标识设备：这是识别执行医学影像采集或处理的物理设备（如 CT 扫描仪、MRI 机器、X 光机、超声设备、工作站等）的主要方式之一。它通常是由设备制造商分配的、在该制造商产品线中唯一的序列号。
    设备溯源：当需要追踪图像来源、进行质量控制、故障排查、维护记录查询或法规审计时，设备序列号是至关重要的信息。它能精确地定位到生成特定图像的那台具体机器。
    数据关联：在 PACS（影像归档与通信系统）、RIS（放射信息系统）或研究数据库中，可以根据设备序列号来筛选、统计或分析来自特定设备的所有影像数据。
     */
    pub device_serial_number: Option<BoundedString<64>>,
    pub software_versions: Option<BoundedString<64>>,
    pub transfer_syntax_uid: SopUidString,
    pub pixel_data_location: Option<BoundedString<512>>,
    pub thumbnail_location: Option<BoundedString<512>>,
    pub sop_class_uid: SopUidString,
    pub image_status: Option<BoundedString<64>>,
    pub space_size: Option<u64>, // 新增字段
    pub created_time: Option<NaiveDateTime>,
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

/// 对 Vec<DicomStateMeta> 进行去重处理


pub fn make_image_info(
    tenant_id: &str,
    dicom_obj: &InMemDicomObject,
) -> Result<DicomImageMeta, DicomParseError> {
    // 必填字段验证 - 确保不为空且长度不超过64
    let patient_id_str = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_ID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Patient ID".to_string()))?;

    let study_uid = dicom_utils::get_text_value(dicom_obj, tags::STUDY_INSTANCE_UID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Study Instance UID".to_string()))?;

    let series_uid = dicom_utils::get_text_value(dicom_obj, tags::SERIES_INSTANCE_UID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Series Instance UID".to_string()))?;

    let sop_uid = dicom_utils::get_text_value(dicom_obj, tags::SOP_INSTANCE_UID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("SOP Instance UID".to_string()))?;

    // study_date_origin 需要确保是 YYYYMMDD 格式
    let study_date_str =
        dicom_utils::get_text_value(dicom_obj, tags::STUDY_DATE).ok_or_else(|| {
            DicomParseError::MissingRequiredField("Study Date text value".to_string())
        })?;

    // 验证格式为 YYYYMMDD
    if study_date_str.len() != 8 || !study_date_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(DicomParseError::InvalidFormat(format!(
            "Study Date must be in YYYYMMDD format, got: {}",
            study_date_str
        )));
    }

    // 图像相关信息
    let instance_number = dicom_utils::get_int_value(dicom_obj, tags::INSTANCE_NUMBER);
    let image_comments = dicom_utils::get_text_value(dicom_obj, tags::IMAGE_COMMENTS)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<256>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert image comments".to_string())
        })?;

    let content_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::CONTENT_DATE)
        .map(|date| {
            let date_str = date.format("%Y%m%d").to_string();
            DicomDateString::try_from(date_str)
        })
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert content date".to_string())
        })?;

    let content_time = dicom_utils::get_text_value(dicom_obj, tags::CONTENT_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert content time".to_string())
        })?;

    let acquisition_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::ACQUISITION_DATE)
        .map(|date| {
            let date_str = date.format("%Y%m%d").to_string();
            DicomDateString::try_from(date_str)
        })
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert acquisition date".to_string())
        })?;

    let acquisition_time = dicom_utils::get_text_value(dicom_obj, tags::ACQUISITION_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert acquisition time".to_string())
        })?;

    let acquisition_date_time =
        dicom_utils::get_datetime_value_dicom(dicom_obj, tags::ACQUISITION_DATE_TIME);

    let image_type = dicom_utils::get_text_value(dicom_obj, tags::IMAGE_TYPE)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<256>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert image type".to_string())
        })?;

    let image_orientation_patient =
        dicom_utils::get_text_value(dicom_obj, tags::IMAGE_ORIENTATION_PATIENT);
    let image_position_patient =
        dicom_utils::get_text_value(dicom_obj, tags::IMAGE_POSITION_PATIENT);

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
    let rescale_type = dicom_utils::get_text_value(dicom_obj, tags::RESCALE_TYPE);

    let window_center = dicom_utils::get_text_value(dicom_obj, tags::WINDOW_CENTER);
    let window_width = dicom_utils::get_text_value(dicom_obj, tags::WINDOW_WIDTH);

    let number_of_frames = dicom_utils::get_tag_value(tags::NUMBER_OF_FRAMES, dicom_obj, 1);

    let acquisition_device_processing_description =
        dicom_utils::get_text_value(dicom_obj, tags::ACQUISITION_DEVICE_PROCESSING_DESCRIPTION)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<256>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert acquisition device processing description".to_string(),
                )
            })?;

    let acquisition_device_processing_code =
        dicom_utils::get_text_value(dicom_obj, tags::ACQUISITION_DEVICE_PROCESSING_CODE)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<256>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert acquisition device processing code".to_string(),
                )
            })?;

    let device_serial_number = dicom_utils::get_text_value(dicom_obj, tags::DEVICE_SERIAL_NUMBER)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert device serial number".to_string())
        })?;

    let software_versions = dicom_utils::get_text_value(dicom_obj, tags::SOFTWARE_VERSIONS)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert software versions".to_string())
        })?;

    let transfer_syntax_uid = dicom_utils::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "1.2.840.10008.1.2".to_string());

    let sop_class_uid = dicom_utils::get_text_value(dicom_obj, tags::SOP_CLASS_UID)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| DicomParseError::MissingRequiredField("SOP Class UID".to_string()))?;

    let image_status = Some(
        BoundedString::<64>::try_from("ACTIVE".to_string()).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert image status".to_string())
        })?,
    );

    // 计算哈希值
    let study_uid_hash = uid_hash::uid_to_u64_deterministic_safe(&study_uid);
    let series_uid_hash = uid_hash::uid_to_u32_deterministic_safe(&study_uid, &series_uid);

    // 时间戳
    let now = chrono::Local::now().naive_local();
    let study_date_origin = DicomDateString::try_from(study_date_str).map_err(|_| {
        DicomParseError::ConversionError("Failed to convert study date origin".to_string())
    })?;

    Ok(DicomImageMeta {
        tenant_id: BoundedString::<64>::try_from(tenant_id.to_string()).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert tenant ID".to_string())
        })?,
        patient_id: BoundedString::<64>::try_from(patient_id_str).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient ID".to_string())
        })?,
        study_uid: SopUidString::try_from(study_uid).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert study UID".to_string())
        })?,
        series_uid: SopUidString::try_from(series_uid).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert series UID".to_string())
        })?,
        sop_uid: SopUidString::try_from(sop_uid).map_err(|_| {
            DicomParseError::ConversionError("Failed to convert SOP UID".to_string())
        })?,
        study_uid_hash,
        series_uid_hash,
        study_date_origin,

        instance_number,
        image_comments,
        content_date,
        content_time,
        acquisition_date,
        acquisition_time,
        acquisition_date_time,
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
        number_of_frames,

        acquisition_device_processing_description,
        acquisition_device_processing_code,
        device_serial_number,
        software_versions,
        transfer_syntax_uid: SopUidString::try_from(transfer_syntax_uid).unwrap(),
        pixel_data_location: None,
        thumbnail_location: None,
        sop_class_uid: SopUidString::try_from(sop_class_uid).map_err(|_| {
            DicomParseError::SopClassUidIsEmpty("SOP Class UID is empty".to_string())
        })?,
        image_status,
        space_size: None,
        created_time: Some(now),
        updated_time: Some(now),
    })
}

pub fn make_state_info(
    tenant_id: &str,
    dicom_obj: &InMemDicomObject,
) -> Result<DicomStateMeta, DicomParseError> {
    // 必填字段验证 - 确保不为空
    // 必填字段验证 - 确保不为空且长度不超过64
    let patient_id_str = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_ID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Patient ID".to_string()))?;

    let study_uid = dicom_utils::get_text_value(dicom_obj, tags::STUDY_INSTANCE_UID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Study Instance UID".to_string()))?;

    let series_uid = dicom_utils::get_text_value(dicom_obj, tags::SERIES_INSTANCE_UID)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Series Instance UID".to_string()))?;
    let acc_num = dicom_utils::get_text_value(dicom_obj, tags::ACCESSION_NUMBER)
        .filter(|v| !v.is_empty() && v.len() <= 16)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Accession Number".to_string()))?;
    // study_date 必须存在且为有效日期
    let study_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::STUDY_DATE)
        .ok_or_else(|| DicomParseError::MissingRequiredField("Study Date".to_string()))?;

    // study_date_origin 需要确保是 YYYYMMDD 格式
    let study_date_str =
        dicom_utils::get_text_value(dicom_obj, tags::STUDY_DATE).ok_or_else(|| {
            DicomParseError::MissingRequiredField("Study Date text value".to_string())
        })?;

    // 验证格式为 YYYYMMDD
    if study_date_str.len() != 8 || !study_date_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(DicomParseError::InvalidFormat(format!(
            "Study Date must be in YYYYMMDD format, got: {}",
            study_date_str
        )));
    }
    let modality = dicom_utils::get_text_value(dicom_obj, tags::MODALITY)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| DicomParseError::MissingRequiredField("Modality".to_string()))?;
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
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient birth time".to_string())
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

    let medical_alerts = dicom_utils::get_text_value(dicom_obj, tags::MEDICAL_ALERTS)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert medical alerts".to_string())
        })?;

    let allergies = dicom_utils::get_text_value(dicom_obj, tags::ALLERGIES)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| DicomParseError::ConversionError("Failed to convert allergies".to_string()))?;

    let pregnancy_status = dicom_utils::get_int_value(dicom_obj, tags::PREGNANCY_STATUS);
    let occupation = dicom_utils::get_text_value(dicom_obj, tags::OCCUPATION)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert occupation".to_string())
        })?;

    let additional_patient_history =
        dicom_utils::get_text_value(dicom_obj, tags::ADDITIONAL_PATIENT_HISTORY)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<512>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert additional patient history".to_string(),
                )
            })?;

    let patient_comments = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_COMMENTS)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<512>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert patient comments".to_string())
        })?;

    // 检查相关信息
    let study_time = dicom_utils::get_text_value(dicom_obj, tags::STUDY_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert study time".to_string())
        })?;

    let study_id = dicom_utils::get_text_value(dicom_obj, tags::STUDY_ID)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| DicomParseError::ConversionError("Failed to convert study ID".to_string()))?;

    let study_description = dicom_utils::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert study description".to_string())
        })?;

    let referring_physician_name =
        dicom_utils::get_text_value(dicom_obj, tags::REFERRING_PHYSICIAN_NAME)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<64>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert referring physician name".to_string(),
                )
            })?;

    let admission_id = dicom_utils::get_text_value(dicom_obj, tags::ADMISSION_ID)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert admission ID".to_string())
        })?;

    let performing_physician_name =
        dicom_utils::get_text_value(dicom_obj, tags::PERFORMING_PHYSICIAN_NAME)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<64>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert performing physician name".to_string(),
                )
            })?;

    let procedure_code_sequence =
        dicom_utils::get_text_value(dicom_obj, tags::PROCEDURE_CODE_SEQUENCE)
            .filter(|v| !v.is_empty())
            .map(|v| BoundedString::<512>::try_from(v))
            .transpose()
            .map_err(|_| {
                DicomParseError::ConversionError(
                    "Failed to convert procedure code sequence".to_string(),
                )
            })?;

    let received_instances = Some(0); // 默认值

    // 序列相关信息

    let series_number = dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER);
    let series_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE);

    let series_time = dicom_utils::get_text_value(dicom_obj, tags::SERIES_TIME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert series time".to_string())
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

    let operators_name = dicom_utils::get_text_value(dicom_obj, tags::OPERATORS_NAME)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<64>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert operators name".to_string())
        })?;

    let number_of_series_related_instances =
        dicom_utils::get_int_value(dicom_obj, tags::NUMBER_OF_SERIES_RELATED_INSTANCES);

    // 计算哈希值
    let study_uid_hash = uid_hash::uid_to_u64_deterministic_safe(&study_uid);
    let series_uid_hash = uid_hash::uid_to_u32_deterministic_safe(&study_uid, &series_uid);

    // 时间戳
    let now = chrono::Local::now().naive_local();
    let study_date_origin = DicomDateString::try_from(study_date_str).unwrap();

    let tenant_id = BoundedString::<64>::try_from(tenant_id.to_string()).unwrap();
    let patient_id = BoundedString::<64>::try_from(patient_id_str).unwrap();
    let study_uid = SopUidString::try_from(study_uid).unwrap();
    let series_uid = SopUidString::try_from(series_uid).unwrap();
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
        medical_alerts,
        allergies,
        pregnancy_status,
        occupation,
        additional_patient_history,
        patient_comments,

        // 检查信息
        study_date,
        study_time,
        accession_number,
        study_id,
        study_description,
        referring_physician_name,
        admission_id,
        performing_physician_name,
        procedure_code_sequence,
        received_instances,

        // 序列信息
        modality,
        series_number,
        series_date,
        series_time,
        series_description,
        body_part_examined,
        protocol_name,
        operators_name,
        number_of_series_related_instances,
        // 时间戳
        created_time: Some(now),
        updated_time: Some(now),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dicom_object::collector::CharacterSetOverride;
    use dicom_object::open_file;
    use rstest::rstest;
    use std::fs;
    use std::path::Path;

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
    #[case("/media/dhz/DCP/DicomTestDataSet/4.90")]

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
                    let result = make_state_info("1234567890", &dicom_obj);

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
                                !state_meta.modality.is_empty(),
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
    #[case("/media/dhz/DCP/DicomTestDataSet/4.90")]

    fn test_make_image_info_with_dicom_files(#[case] dicom_dir: &str) {
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
                    let result = make_image_info("1234567890", &dicom_obj);

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
                                !state_meta.sop_uid.as_str().is_empty(),
                                "SOP UID should not be empty in file: {:?}",
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

    // 递归收集目录及其子目录中的所有.dcm文件
    fn collect_dicom_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 递归遍历子目录
                    collect_dicom_files(&path, files);
                } else if path.extension().map_or(false, |ext| ext == "dcm") {
                    // 添加.dcm文件到列表
                    files.push(path);
                }
            }
        }
    }

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
