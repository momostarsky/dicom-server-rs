use crate::dicom_utils;

use crate::storage_config::hash_uid;
use crate::utils::get_current_time;
use chrono::NaiveDate;
use database::dicom_dbtype::{BoundedString, DicomDateString};
use database::dicom_meta::{DicomImageMeta, DicomStateMeta};
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;
use serde::{Deserialize, Serialize};

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

struct DicomCommonMeta {
    patient_id: String,
    study_uid: String,
    series_uid: String,
    sop_uid: String,
    study_date: NaiveDate,
    study_date_str: String,
}

impl DicomCommonMeta {
    const KEY_TAGS: &'static [dicom_core::Tag] = &[
        tags::PATIENT_ID,
        tags::STUDY_INSTANCE_UID,
        tags::SERIES_INSTANCE_UID,
        tags::SOP_INSTANCE_UID,
        tags::STUDY_DATE,
    ];
    fn extract_from_dicom(dicom_obj: &InMemDicomObject) -> Result<Self, DicomParseError> {
        let mut patient_id_str: Option<String> = None;
        let mut study_uid_str: Option<String> = None;
        let mut series_uid_str: Option<String> = None;
        let mut sop_uid_str: Option<String> = None;
        let mut study_date: Option<(String, NaiveDate)> = None;

        // 验证所有必需的标签是否存在且符合长度要求
        for &tag in Self::KEY_TAGS {
            let value = match dicom_utils::get_text_value(dicom_obj, tag) {
                Some(value) => value,
                None => {
                    // 标签不存在
                    let tag_name = match tag {
                        tags::PATIENT_ID => "PATIENT_ID",
                        tags::STUDY_INSTANCE_UID => "STUDY_INSTANCE_UID",
                        tags::SERIES_INSTANCE_UID => "SERIES_INSTANCE_UID",
                        tags::SOP_INSTANCE_UID => "SOP_INSTANCE_UID",
                        tags::STUDY_DATE => "STUDY_DATE",
                        _ => "UNKNOWN_TAG",
                    };
                    return Err(DicomParseError::MissingRequiredField(tag_name.to_string()));
                }
            };

            // 检查值是否为空
            if value.is_empty() {
                return Err(DicomParseError::MissingRequiredField(format!(
                    "Tag {:?} has empty value",
                    tag
                )));
            }

            // 检查特定标签的长度限制
            let max_length = match tag {
                tags::PATIENT_ID => 64,
                tags::STUDY_INSTANCE_UID => 64,
                tags::SERIES_INSTANCE_UID => 64,
                tags::SOP_INSTANCE_UID => 64,
                tags::STUDY_DATE => 8, // YYYYMMDD format
                _ => usize::MAX,       // 其他标签不限制长度
            };
            if value.len() > max_length {
                return Err(DicomParseError::InvalidFormat(format!(
                    "Tag {:?} value exceeds maximum length {}: {}",
                    tag, max_length, value
                )));
            }

            // 在验证STUDY_DATE时保存解析结果
            if tag == tags::STUDY_DATE {
                match NaiveDate::parse_from_str(&value, "%Y%m%d") {
                    Ok(date) => {
                        // 保存解析结果供后续使用
                        study_date = Some((value.clone(), date));
                    }
                    Err(_) => {
                        return Err(DicomParseError::InvalidDateFormat(format!(
                            "Tag {:?} value is not valid date format:YYYYMMDD {}",
                            tag, value
                        )));
                    }
                }
            }

            // 赋值给相应变量
            match tag {
                tags::PATIENT_ID => patient_id_str = Some(value),
                tags::STUDY_INSTANCE_UID => study_uid_str = Some(value),
                tags::SERIES_INSTANCE_UID => series_uid_str = Some(value),
                tags::SOP_INSTANCE_UID => sop_uid_str = Some(value),
                _ => {}
            }
        }

        // 安全解包Option值
        let patient_id = patient_id_str.unwrap();
        let study_uid = study_uid_str.unwrap();
        let series_uid = series_uid_str.unwrap();
        let sop_uid = sop_uid_str.unwrap();
        // 解包时
        let (study_date_str_value, study_date_v) = study_date.unwrap();

        Ok(DicomCommonMeta {
            patient_id,
            study_uid,
            series_uid,
            sop_uid,
            study_date: study_date_v,
            study_date_str: study_date_str_value,
        })
    }
}
// 在文件顶部添加辅助函数
fn convert_bounded_string<const N: usize>(
    value: String,
) -> Result<BoundedString<N>, DicomParseError> {
    BoundedString::<N>::try_from(value.clone()).map_err(|_| {
        DicomParseError::ConversionError(format!("Failed to convert string:{}", &value))
    })
}

/// 对 Vec<DicomStateMeta> 进行去重处理

pub fn make_image_info(
    tenant_id: &str,
    dicom_obj: &InMemDicomObject,
    fsize: Option<i64>,
) -> Result<DicomImageMeta, DicomParseError> {
    // 使用公共提取器获取基本信息
    let common_meta = DicomCommonMeta::extract_from_dicom(dicom_obj)?;
    // 图像相关信息
    let instance_number = dicom_utils::get_int_value(dicom_obj, tags::INSTANCE_NUMBER);

    let content_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::CONTENT_DATE);

    let content_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::CONTENT_TIME);

    let image_type = dicom_utils::get_text_value(dicom_obj, tags::IMAGE_TYPE)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<128>)
        .transpose()?;

    let image_orientation_patient =
        dicom_utils::get_text_value(dicom_obj, tags::IMAGE_ORIENTATION_PATIENT)
            .filter(|v| !v.is_empty())
            .map(convert_bounded_string::<128>)
            .transpose()?;

    let image_position_patient =
        dicom_utils::get_text_value(dicom_obj, tags::IMAGE_POSITION_PATIENT)
            .filter(|v| !v.is_empty())
            .map(convert_bounded_string::<64>)
            .transpose()?;

    let slice_thickness = dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_THICKNESS);
    let spacing_between_slices =
        dicom_utils::get_decimal_value(dicom_obj, tags::SPACING_BETWEEN_SLICES);
    let slice_location = dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_LOCATION);

    let samples_per_pixel = dicom_utils::get_int_value(dicom_obj, tags::SAMPLES_PER_PIXEL);
    let photometric_interpretation =
        dicom_utils::get_text_value(dicom_obj, tags::PHOTOMETRIC_INTERPRETATION)
            .filter(|v| !v.is_empty())
            .map(convert_bounded_string::<32>)
            .transpose()?;

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
        .map(convert_bounded_string::<64>)
        .transpose()?;

    let window_center = dicom_utils::get_text_value(dicom_obj, tags::WINDOW_CENTER)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<64>)
        .transpose()?;

    let window_width = dicom_utils::get_text_value(dicom_obj, tags::WINDOW_WIDTH)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<64>)
        .transpose()?;

    let transfer_syntax_uid = dicom_utils::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "1.2.840.10008.1.2".to_string());

    let sop_class_uid = dicom_utils::get_text_value(dicom_obj, tags::SOP_CLASS_UID)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| DicomParseError::MissingRequiredField("SOP Class UID".to_string()))?;

    let image_status = Some(BoundedString::<32>::make_str("ACTIVE"));

    // 计算哈希值
    let study_uid_hash = BoundedString::<20>::make(hash_uid(&common_meta.study_uid));
    let series_uid_hash = BoundedString::<20>::make(hash_uid(&common_meta.series_uid));

    let space_size = fsize;
    // 时间戳
    let now = get_current_time();

    Ok(DicomImageMeta {
        tenant_id: BoundedString::<64>::make(tenant_id.to_string()),
        patient_id: BoundedString::<64>::make(common_meta.patient_id),
        study_uid: BoundedString::<64>::make(common_meta.study_uid),
        series_uid: BoundedString::<64>::make(common_meta.series_uid),
        sop_uid: BoundedString::<64>::make(common_meta.sop_uid),
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

        transfer_syntax_uid: BoundedString::<64>::make(transfer_syntax_uid),
        pixel_data_location: None,
        thumbnail_location: None,
        sop_class_uid: BoundedString::<64>::make(sop_class_uid),
        image_status,
        space_size,
        created_time: now,
        updated_time: now,
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
        .unwrap_or_else(|| format!("ACC{}", make_crc32(tenant_id, msg_study_uid))); // 当为空时设置默认值"X12333"
    let modality = dicom_utils::get_text_value(dicom_obj, tags::MODALITY)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<16>)
        .transpose()?;

    // 患者相关信息
    let patient_name = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_NAME)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<64>)
        .transpose()?;

    let patient_sex = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_SEX)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<1>)
        .transpose()?;

    let patient_birth_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::PATIENT_BIRTH_DATE);

    let patient_birth_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::PATIENT_BIRTH_TIME);

    // 患者其他信息
    let patient_age = dicom_utils::get_text_value(dicom_obj, tags::PATIENT_AGE)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<16>)
        .transpose()?;

    let patient_size = dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_SIZE);
    let patient_weight = dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_WEIGHT);

    let study_date = common_meta.study_date;
    // 检查相关信息
    let study_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::STUDY_TIME);

    let study_id = dicom_utils::get_text_value(dicom_obj, tags::STUDY_ID)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<16>)
        .transpose()?;

    let study_description = dicom_utils::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<64>)
        .transpose()?;
    // 序列相关信息

    let series_number = dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER);
    let series_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE);

    let series_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::SERIES_TIME);

    let series_description = dicom_utils::get_text_value(dicom_obj, tags::SERIES_DESCRIPTION)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<256>)
        .transpose()?;

    let body_part_examined = dicom_utils::get_text_value(dicom_obj, tags::BODY_PART_EXAMINED)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<64>)
        .transpose()?;

    let protocol_name = dicom_utils::get_text_value(dicom_obj, tags::PROTOCOL_NAME)
        .filter(|v| !v.is_empty())
        .map(convert_bounded_string::<64>)
        .transpose()?;

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
    // 计算哈希值
    let study_uid_hash = BoundedString::<20>::make_str(&hash_uid(&common_meta.study_uid));
    let series_uid_hash = BoundedString::<20>::make_str(&hash_uid(&common_meta.series_uid));

    // 时间戳
    let now = chrono::Local::now().naive_local();
    let study_date_origin = DicomDateString::from_str(&common_meta.study_date_str).unwrap();

    let tenant_id = BoundedString::<64>::make_str(tenant_id);
    let patient_id = BoundedString::<64>::make(common_meta.patient_id);
    let study_uid = BoundedString::<64>::make(common_meta.study_uid);
    let series_uid = BoundedString::<64>::make(common_meta.series_uid);
    //
    //https://dicom.nema.org/medical/dicom/current/output/chtml/part05/sect_6.2.html
    //
    let accession_number = BoundedString::<16>::make(acc_num);

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
