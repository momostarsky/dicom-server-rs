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

#[derive(Debug)]
struct DicomCommonMeta {
    patient_id: String,
    study_uid: String,
    series_uid: String,
    sop_uid: String,
    sop_class_uid: String,
    study_date: NaiveDate,
    study_date_str: String,
}

impl DicomCommonMeta {
    const KEY_TAGS: &'static [dicom_core::Tag] = &[
        tags::PATIENT_ID,
        tags::STUDY_INSTANCE_UID,
        tags::SERIES_INSTANCE_UID,
        tags::SOP_INSTANCE_UID,
        tags::SOP_CLASS_UID,
        tags::STUDY_DATE,
    ];
    fn extract_from_dicom(dicom_obj: &InMemDicomObject) -> Result<Self, DicomParseError> {
        let mut patient_id_str: String = String::new();
        let mut study_uid_str: String = String::new();
        let mut series_uid_str: String = String::new();
        let mut sop_uid_str: String = String::new();
        let mut sop_class_uid_str: String = String::new();
        let mut study_date_parsed: Option<(String, NaiveDate)> = None;

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
                        tags::SOP_CLASS_UID => "SOP_CLASS_UID",
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
                tags::SOP_CLASS_UID => 64,
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
                        study_date_parsed = Some((value, date));
                    }
                    Err(_) => {
                        return Err(DicomParseError::InvalidDateFormat(format!(
                            "Tag {:?} value is not valid date format:YYYYMMDD {}",
                            tag, value
                        )));
                    }
                }
            } else {
                // 直接赋值给相应变量
                match tag {
                    tags::PATIENT_ID => patient_id_str = value,
                    tags::STUDY_INSTANCE_UID => study_uid_str = value,
                    tags::SERIES_INSTANCE_UID => series_uid_str = value,
                    tags::SOP_INSTANCE_UID => sop_uid_str = value,
                    tags::SOP_CLASS_UID => sop_class_uid_str = value,
                    _ => {}
                }
            }
        }

        // 解包study_date
        let (study_date_str_value, study_date_v) = study_date_parsed.ok_or(
            DicomParseError::MissingRequiredField("STUDY_DATE".to_string()),
        )?;

        Ok(DicomCommonMeta {
            patient_id: patient_id_str,
            study_uid: study_uid_str,
            series_uid: series_uid_str,
            sop_uid: sop_uid_str,
            sop_class_uid: sop_class_uid_str,
            study_date: study_date_v,
            study_date_str: study_date_str_value,
        })
    }
}

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

    let image_type = dicom_utils::get_bounder_string::<128>(dicom_obj, tags::IMAGE_TYPE);

    let image_orientation_patient =
        dicom_utils::get_bounder_string::<128>(dicom_obj, tags::IMAGE_ORIENTATION_PATIENT);

    let image_position_patient =
        dicom_utils::get_bounder_string::<64>(dicom_obj, tags::IMAGE_POSITION_PATIENT);

    let slice_thickness = dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_THICKNESS);
    let spacing_between_slices =
        dicom_utils::get_decimal_value(dicom_obj, tags::SPACING_BETWEEN_SLICES);
    let slice_location = dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_LOCATION);

    let samples_per_pixel = dicom_utils::get_int_value(dicom_obj, tags::SAMPLES_PER_PIXEL);
    let photometric_interpretation =
        dicom_utils::get_bounder_string::<32>(dicom_obj, tags::PHOTOMETRIC_INTERPRETATION);

    let width = dicom_utils::get_int_value(dicom_obj, tags::ROWS);
    let columns = dicom_utils::get_int_value(dicom_obj, tags::COLUMNS);
    let bits_allocated = dicom_utils::get_int_value(dicom_obj, tags::BITS_ALLOCATED);
    let bits_stored = dicom_utils::get_int_value(dicom_obj, tags::BITS_STORED);
    let high_bit = dicom_utils::get_int_value(dicom_obj, tags::HIGH_BIT);
    let pixel_representation = dicom_utils::get_int_value(dicom_obj, tags::PIXEL_REPRESENTATION);

    let rescale_intercept = dicom_utils::get_decimal_value(dicom_obj, tags::RESCALE_INTERCEPT);
    let rescale_slope = dicom_utils::get_decimal_value(dicom_obj, tags::RESCALE_SLOPE);

    let rescale_type = dicom_utils::get_bounder_string::<64>(dicom_obj, tags::RESCALE_TYPE);

    let window_center = dicom_utils::get_bounder_string::<64>(dicom_obj, tags::WINDOW_CENTER);

    let window_width = dicom_utils::get_bounder_string::<64>(dicom_obj, tags::WINDOW_WIDTH);

    let transfer_syntax_uid =
        dicom_utils::get_bounder_string::<64>(dicom_obj, tags::TRANSFER_SYNTAX_UID)
            .unwrap_or_else(|| BoundedString::<64>::make_str("1.2.840.10008.1.2"));

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

        transfer_syntax_uid,
        pixel_data_location: None,
        thumbnail_location: None,
        sop_class_uid: BoundedString::<64>::make(common_meta.sop_class_uid),
        image_status,
        space_size,
        created_time: now,
        updated_time: now,
    })
}


pub fn make_state_info(
    tenant_id: &str,
    dicom_obj: &InMemDicomObject,
) -> Result<DicomStateMeta, DicomParseError> {
    // 必填字段验证 - 确保不为空
    // 必填字段验证 - 确保不为空且长度不超过64
    // 使用公共提取器获取基本信息
    let common_meta = DicomCommonMeta::extract_from_dicom(dicom_obj)?;

    let acc_num = dicom_utils::get_bounder_string::<16>(dicom_obj, tags::ACCESSION_NUMBER);

    let modality = dicom_utils::get_bounder_string::<16>(dicom_obj, tags::MODALITY);

    // 患者相关信息
    let patient_name = dicom_utils::get_bounder_string::<64>(dicom_obj, tags::PATIENT_NAME);

    let patient_sex = dicom_utils::get_bounder_string::<1>(dicom_obj, tags::PATIENT_SEX);

    let patient_birth_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::PATIENT_BIRTH_DATE);

    let patient_birth_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::PATIENT_BIRTH_TIME);

    // 患者其他信息
    let patient_age = dicom_utils::get_bounder_string::<16>(dicom_obj, tags::PATIENT_AGE);

    let patient_size = dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_SIZE);
    let patient_weight = dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_WEIGHT);

    let study_date = common_meta.study_date;
    // 检查相关信息
    let study_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::STUDY_TIME);

    let study_id = dicom_utils::get_bounder_string::<16>(dicom_obj, tags::STUDY_ID);

    let study_description =
        dicom_utils::get_bounder_string::<64>(dicom_obj, tags::STUDY_DESCRIPTION);
    // 序列相关信息

    let series_number = dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER);
    let series_date = dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE);

    let series_time = dicom_utils::get_time_value_dicom(dicom_obj, tags::SERIES_TIME);

    let series_description =
        dicom_utils::get_bounder_string::<256>(dicom_obj, tags::SERIES_DESCRIPTION);

    let body_part_examined =
        dicom_utils::get_bounder_string::<64>(dicom_obj, tags::BODY_PART_EXAMINED);

    let protocol_name = dicom_utils::get_bounder_string::<64>(dicom_obj, tags::PROTOCOL_NAME);

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
        accession_number: acc_num,
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
    use chrono::NaiveDate;
    use dicom_core::{DataElement, PrimitiveValue, VR};
    use dicom_dictionary_std::tags;
    use dicom_object::collector::CharacterSetOverride;

    fn create_test_dicom_object_for_meta() -> InMemDicomObject {
        // 创建一个完整的DICOM对象用于测试DicomCommonMeta和make_*函数
        let obj = InMemDicomObject::from_element_iter([
            // 必需字段
            DataElement::new(tags::PATIENT_ID, VR::LO, PrimitiveValue::from("PATIENT123")),
            DataElement::new(
                tags::STUDY_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9"),
            ),
            DataElement::new(
                tags::SERIES_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9.1"),
            ),
            DataElement::new(
                tags::SOP_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9.1.1"),
            ),
            DataElement::new(tags::STUDY_DATE, VR::DA, PrimitiveValue::from("20230115")),
            // 图像相关信息
            DataElement::new(tags::INSTANCE_NUMBER, VR::IS, PrimitiveValue::from("1")),
            DataElement::new(tags::CONTENT_DATE, VR::DA, PrimitiveValue::from("20230115")),
            DataElement::new(tags::CONTENT_TIME, VR::TM, PrimitiveValue::from("120000")),
            DataElement::new(
                tags::IMAGE_TYPE,
                VR::CS,
                PrimitiveValue::from("ORIGINAL\\PRIMARY"),
            ),
            DataElement::new(
                tags::TRANSFER_SYNTAX_UID,
                VR::UI,
                PrimitiveValue::from("1.2.840.10008.1.2.1"),
            ),
            DataElement::new(
                tags::SOP_CLASS_UID,
                VR::UI,
                PrimitiveValue::from("1.2.840.10008.5.1.4.1.1.2"),
            ),
            // 患者信息
            DataElement::new(tags::PATIENT_NAME, VR::PN, PrimitiveValue::from("Doe^John")),
            DataElement::new(tags::PATIENT_SEX, VR::CS, PrimitiveValue::from("M")),
            DataElement::new(
                tags::PATIENT_BIRTH_DATE,
                VR::DA,
                PrimitiveValue::from("19800101"),
            ),
            // 检查信息
            DataElement::new(
                tags::ACCESSION_NUMBER,
                VR::SH,
                PrimitiveValue::from("ACC123456789"),
            ),
            DataElement::new(tags::STUDY_TIME, VR::TM, PrimitiveValue::from("093000")),
            DataElement::new(tags::STUDY_ID, VR::SH, PrimitiveValue::from("STUDY123")),
            DataElement::new(
                tags::STUDY_DESCRIPTION,
                VR::LO,
                PrimitiveValue::from("胸部CT检查"),
            ),
            // 序列信息
            DataElement::new(tags::MODALITY, VR::CS, PrimitiveValue::from("CT")),
            DataElement::new(tags::SERIES_NUMBER, VR::IS, PrimitiveValue::from("1")),
            DataElement::new(tags::SERIES_DATE, VR::DA, PrimitiveValue::from("20230115")),
            DataElement::new(tags::SERIES_TIME, VR::TM, PrimitiveValue::from("093000")),
            DataElement::new(
                tags::SERIES_DESCRIPTION,
                VR::LO,
                PrimitiveValue::from("常规扫描"),
            ),
            DataElement::new(
                tags::BODY_PART_EXAMINED,
                VR::CS,
                PrimitiveValue::from("CHEST"),
            ),
            DataElement::new(
                tags::PROTOCOL_NAME,
                VR::LO,
                PrimitiveValue::from("胸部平扫"),
            ),
        ]);

        obj
    }

    #[test]
    fn test_extract_from_dicom_success() {
        let obj = create_test_dicom_object_for_meta();
        let result = DicomCommonMeta::extract_from_dicom(&obj);

        assert!(result.is_ok());
        let common_meta = result.unwrap();

        assert_eq!(common_meta.patient_id, "PATIENT123");
        assert_eq!(common_meta.study_uid, "1.2.3.4.5.6.7.8.9");
        assert_eq!(common_meta.series_uid, "1.2.3.4.5.6.7.8.9.1");
        assert_eq!(common_meta.sop_uid, "1.2.3.4.5.6.7.8.9.1.1");
        assert_eq!(common_meta.study_date_str, "20230115");
        assert_eq!(
            common_meta.study_date,
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()
        );
    }

    #[test]
    fn test_extract_from_dicom_missing_required_field() {
        // 创建缺少必需字段的DICOM对象
        let obj = InMemDicomObject::from_element_iter([DataElement::new(
            tags::PATIENT_ID,
            VR::LO,
            PrimitiveValue::from("PATIENT123"),
        )]);

        let result = DicomCommonMeta::extract_from_dicom(&obj);
        assert!(result.is_err());

        match result.unwrap_err() {
            DicomParseError::MissingRequiredField(field) => {
                assert_eq!(field, "STUDY_INSTANCE_UID");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
    }

    #[test]
    fn test_extract_from_dicom_invalid_date_format() {
        let obj = InMemDicomObject::from_element_iter([
            DataElement::new(tags::PATIENT_ID, VR::LO, PrimitiveValue::from("PATIENT123")),
            DataElement::new(
                tags::STUDY_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9"),
            ),
            DataElement::new(
                tags::SERIES_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9.1"),
            ),
            DataElement::new(
                tags::SOP_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9.1.1"),
            ),
            DataElement::new(
                tags::SOP_CLASS_UID,
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5.6.7.8.9.909"),
            ),
            DataElement::new(tags::STUDY_DATE, VR::DA, PrimitiveValue::from("INVALID")),
        ]);

        let result = DicomCommonMeta::extract_from_dicom(&obj);
        assert!(result.is_err());

        match result.unwrap_err() {
            DicomParseError::InvalidDateFormat(msg) => {
                // 修改断言以匹配实际的错误消息格式
                assert!(msg.contains("value is not valid date format"));
                assert!(msg.contains("INVALID"));
            }
            _ => panic!("Expected InvalidDateFormat error"),
        }
    }

    #[test]
    fn test_make_image_info_success() {
        use dicom_test_files::path;

        let liver = path("pydicom/liver.dcm").unwrap();
        // then open the file as you will (e.g. using DICOM-rs)
        match dicom_object::OpenFileOptions::new()
            .charset_override(CharacterSetOverride::AnyVr)
            .read_until(tags::PIXEL_DATA)
            .open_file(liver.to_str().unwrap())
        {
            Ok(dicom_obj) => {
                let result = make_image_info("tenant1", &dicom_obj, Some(1024));
                assert!(result.is_ok());

                // 将结果输出为 JSON 格式
                let image_meta = result.unwrap();
                let json_output = serde_json::to_string_pretty(&image_meta).unwrap();
                println!("{}", json_output);
            }
            Err(err) => {
                println!("Failed to open DICOM file: {}", err);
            }
        };
    }

    #[test]
    fn test_make_state_info_success() {
        let obj = create_test_dicom_object_for_meta();
        let result = make_state_info("tenant1", &obj);

        assert!(result.is_ok());
        let state_meta = result.unwrap();

        assert_eq!(state_meta.tenant_id.as_str(), "tenant1");
        assert_eq!(state_meta.patient_id.as_str(), "PATIENT123");
        assert_eq!(state_meta.study_uid.as_str(), "1.2.3.4.5.6.7.8.9");
        assert_eq!(state_meta.series_uid.as_str(), "1.2.3.4.5.6.7.8.9.1");
        assert_eq!(
            state_meta.patient_name.as_ref().unwrap().as_str(),
            "Doe^John"
        );
        assert_eq!(state_meta.modality.as_ref().unwrap().as_str(), "CT");

    }


}
