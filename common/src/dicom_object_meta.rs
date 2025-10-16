use crate::string_ext::{BoundedString, DicomDateString, SopUidString, UuidString};
use crate::{dicom_utils, uid_hash};
use chrono::NaiveDateTime;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PerisitStatus {
    /// 默认状态
    Unknown,
    /// 解析失败
    ParseTagFailed,
    /// 写入数据库失败
    WriteToDatabaseFailed,
    /// 写入数据库成功
    WriteToDatabaseSuccess,
}

/// DicomParseMeta 用于记录DICOM文件元数据入库前的解析日志.
/// 包含了所有必要的元数据字段.每一个DicomParseMeta实例标识解析一个DICOM文件.并成功提取元数据.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomPerisitMeta {
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
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "persist_state")]
    pub persist_state: PerisitStatus,
    #[serde(rename = "persist_time")]
    pub persist_time: NaiveDateTime,
    #[serde(rename = "persist_message")]
    pub persist_message: BoundedString<512>,
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
    pub accession_number: Option<BoundedString<16>>,
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
pub enum DicomParseError {
    MissingRequiredField(String),
    InvalidFormat(String),
    ConversionError(String),
    // 可以根据需要添加其他错误类型
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

    let accession_number = dicom_utils::get_text_value(dicom_obj, tags::ACCESSION_NUMBER)
        .filter(|v| !v.is_empty())
        .map(|v| BoundedString::<16>::try_from(v))
        .transpose()
        .map_err(|_| {
            DicomParseError::ConversionError("Failed to convert accession number".to_string())
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
    let study_date_origin = DicomDateString::try_from(study_date_str).map_err(|_| {
        DicomParseError::ConversionError("Failed to convert study date origin".to_string())
    })?;

    let tenant_id = BoundedString::<64>::try_from(tenant_id.to_string()).unwrap();

    let patient_id = BoundedString::<64>::try_from(patient_id_str).unwrap();
    let study_uid = SopUidString::try_from(study_uid).unwrap();
    let series_uid = SopUidString::try_from(series_uid).unwrap();
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
    use std::fs;
    use std::path::Path;
     use rstest::rstest;

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
            println!("Processing file {}/{}: {:?}", index + 1, dicom_files.len(), path);

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


    #[test]
    fn test_make_state_info_with_sample_files() {
        let dicom_dir = "/media/dhz/DCP/DicomTestDataSet";

        // 检查目录是否存在
        if !Path::new(dicom_dir).exists() {
            println!("DICOM test directory not found: {}", dicom_dir);
            return;
        }

        // 收集所有.dcm文件
        let mut dicom_files = Vec::new();
        collect_dicom_files(Path::new(dicom_dir), &mut dicom_files);

        // 只测试前5个文件以避免测试时间过长
        let max_files = 5.min(dicom_files.len());
        let sample_files = &dicom_files[..max_files];

        for (index, path) in sample_files.iter().enumerate() {
            println!("Processing file {}/{}: {:?}", index + 1, sample_files.len(), path);

            // 尝试打开DICOM文件
            match open_file(path) {
                Ok(dicom_obj) => {
                    // 测试make_state_info函数
                    let result = make_state_info("1234567890", &dicom_obj);

                    match result {
                        Ok(state_meta) => {
                            // 验证关键字段长度限制
                            assert!(
                                state_meta.patient_id.as_str().len() <= 64,
                                "Patient ID exceeds 64 characters in file: {:?}",
                                path
                            );
                            assert!(
                                state_meta.study_uid.as_str().len() <= 64,
                                "Study UID exceeds 64 characters in file: {:?}",
                                path
                            );
                            assert!(
                                state_meta.series_uid.as_str().len() <= 64,
                                "Series UID exceeds 64 characters in file: {:?}",
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
                    panic!("Failed to open DICOM file {:?}: {:?}", path, e);
                }
            }
        }

        if dicom_files.is_empty() {
            println!("No .dcm files found in directory: {}", dicom_dir);
        } else {
            println!("Successfully processed {} DICOM files ({} sampled)", dicom_files.len(), max_files);
        }
    }
}
