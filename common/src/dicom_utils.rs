use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dicom_core::Tag;
use dicom_object::InMemDicomObject;

pub fn get_text_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
    dicom_obj
        .element(tag)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim_end_matches(|c| c == ' ' || c == '\0').to_string()) // 正确处理尾部空格和\0字符
}

pub fn get_date_value_dicom(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<NaiveDate> {
    dicom_obj.element(tag).ok().and_then(|e| {
        e.to_date().ok().and_then(|date| {
            let year = *date.year() as i32;
            let month = date.month().map(|m| *m as u32).unwrap_or(1);
            let day = date.day().map(|d| *d as u32).unwrap_or(1);
            NaiveDate::from_ymd_opt(year, month, day)
        })
    })
}

pub fn get_time_value_dicom(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<NaiveTime> {
    dicom_obj.element(tag).ok().and_then(|e| {
        e.to_time().ok().and_then(|date| {
            let hour = *date.hour() as u32;
            let minute = date.minute().map(|m| *m as u32).unwrap_or(0);
            let second = date.second().map(|s| *s as u32).unwrap_or(0);
            let millisecond = date.millisecond().unwrap_or(0);
            NaiveTime::from_hms_milli_opt(hour, minute, second, millisecond)
        })
    })

}

pub fn get_datetime_value_dicom(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<NaiveDateTime> {
    dicom_obj.element(tag).ok().and_then(|e| {
        e.to_datetime().ok().and_then(|datetime| {
            let date = datetime.date();
            let time = datetime.time();
            let year = *date.year() as i32;
            let month = date.month().map(|m| *m as u32).unwrap_or(1);
            let day = date.day().map(|d| *d as u32).unwrap_or(1);

            NaiveDate::from_ymd_opt(year, month, day).and_then(|d| {
                let (hour, minute, second, millisecond) = if let Some(time) = time {
                    (
                        *time.hour() as u32,
                        time.minute().map(|m| *m as u32).unwrap_or(0),
                        time.second().map(|s| *s as u32).unwrap_or(0),
                        time.millisecond().unwrap_or(0),
                    )
                } else {
                    (0, 0, 0, 0)
                };

                NaiveTime::from_hms_milli_opt(hour, minute, second, millisecond)
                    .map(|t| NaiveDateTime::new(d, t))
            })
        })
    })
}

pub fn get_int_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<i32> {
    dicom_obj.element(tag).ok().and_then(|e| e.to_int().ok())
}

pub fn get_decimal_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<f64> {
    dicom_obj
        .element(tag)
        .ok()
        .and_then(|e| e.to_float64().ok())
}

pub fn get_tag_value<T>(tag: Tag, obj: &InMemDicomObject, def_value: T) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    obj.element_opt(tag)
        .ok()
        .flatten()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim_end_matches(|c| c == ' ' || c == '\0').to_string()) // 正确处理尾部空格和\0字符
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(def_value)
}

pub fn get_tag_values<T>(tag: Tag, obj: &InMemDicomObject) -> Vec<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    obj.element_opt(tag)
        .ok()
        .flatten()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim_end_matches(|c| c == ' ' || c == '\0').to_string()) // 正确处理尾部空格和\0字符
        .and_then(|s| {
            let mut result = vec![];
            for s in s.trim_end().split("\\") {
                if let Ok(v) = s.parse::<T>() {
                    result.push(v);
                }
            }
            Some(result)
        })
        .unwrap_or_else(|| vec![])
}


 #[cfg(test)]
mod tests {
     use super::*;
     use chrono::{NaiveDate, NaiveTime};
     use dicom_core::{DataElement, PrimitiveValue, VR};
     use dicom_dictionary_std::tags;

     fn create_test_dicom_object() -> InMemDicomObject {
        // 构建一个包含特定数据元素的 InMemDicomObject
        let obj = InMemDicomObject::from_element_iter([
            DataElement::new(
                tags::PATIENT_NAME, // 使用标准标签，例如病人姓名
                VR::PN,
                PrimitiveValue::from("Doe^John"),
            ),
            DataElement::new(
                tags::PATIENT_ID, // 例如病人 ID
                VR::LO,
                PrimitiveValue::from("12345"),
            ),
            DataElement::new(
                tags::STUDY_DATE, // 例如检查日期
                VR::DA,
                PrimitiveValue::from("20251125"),
            ),
            DataElement::new(
                tags::STUDY_TIME, // 检查时间
                VR::TM,
                PrimitiveValue::from("143025.123"),
            ),
            DataElement::new(
                tags::ACCESSION_NUMBER, // 检查号
                VR::SH,
                PrimitiveValue::from("A123456789"),
            ),
            DataElement::new(
                tags::INSTANCE_NUMBER, // 实例号
                VR::IS,
                PrimitiveValue::from("1"),
            ),
            DataElement::new(
                tags::SLICE_THICKNESS, // 层厚
                VR::DS,
                PrimitiveValue::from("1.5"),
            ),
            DataElement::new(
                tags::PATIENT_BIRTH_DATE, // 病人出生日期
                VR::DA,
                PrimitiveValue::from("19900101"),
            ),
        ]);

        obj
    }

    #[test]
    fn test_get_text_value() {
        let obj = create_test_dicom_object();

        // 测试存在的文本值
        let patient_name = get_text_value(&obj, tags::PATIENT_NAME);
        assert_eq!(patient_name, Some("Doe^John".to_string()));

        // 测试不存在的文本值
        let modality = get_text_value(&obj, tags::MODALITY);
        assert_eq!(modality, None);
    }

    #[test]
    fn test_get_int_value() {
        let obj = create_test_dicom_object();

        // 测试存在的整数值
        let instance_number = get_int_value(&obj, tags::INSTANCE_NUMBER);
        assert_eq!(instance_number, Some(1));

        // 测试不存在的整数值
        let series_number = get_int_value(&obj, tags::SERIES_NUMBER);
        assert_eq!(series_number, None);
    }

    #[test]
    fn test_get_decimal_value() {
        let obj = create_test_dicom_object();

        // 测试存在的浮点数值
        let slice_thickness = get_decimal_value(&obj, tags::SLICE_THICKNESS);
        assert_eq!(slice_thickness, Some(1.5));

        // 测试不存在的浮点数值
        let pixel_spacing = get_decimal_value(&obj, tags::PIXEL_SPACING);
        assert_eq!(pixel_spacing, None);
    }

    #[test]
    fn test_get_date_value_dicom() {
        let obj = create_test_dicom_object();

        // 测试存在的日期值
        let study_date = get_date_value_dicom(&obj, tags::STUDY_DATE);
        let expected_date = NaiveDate::from_ymd_opt(2025, 11, 25).unwrap();
        assert_eq!(study_date, Some(expected_date));

        // 测试不存在的日期值
        let series_date = get_date_value_dicom(&obj, tags::SERIES_DATE);
        assert_eq!(series_date, None);
    }

    #[test]
    fn test_get_time_value_dicom() {
        let obj = create_test_dicom_object();

        // 测试存在的时间值
        let study_time = get_time_value_dicom(&obj, tags::STUDY_TIME);
        let expected_time = NaiveTime::from_hms_milli_opt(14, 30, 25, 123).unwrap();
        assert_eq!(study_time, Some(expected_time));

        // 测试不存在的时间值
        let series_time = get_time_value_dicom(&obj, tags::SERIES_TIME);
        assert_eq!(series_time, None);
    }

    #[test]
    fn test_get_tag_value() {
        let obj = create_test_dicom_object();

        // 测试存在的标签值
        let instance_number = get_tag_value(tags::INSTANCE_NUMBER, &obj, 0i32);
        assert_eq!(instance_number, 1);

        // 测试不存在的标签值，应返回默认值
        let series_number = get_tag_value(tags::SERIES_NUMBER, &obj, 99i32);
        assert_eq!(series_number, 99);
    }

    #[test]
    fn test_get_tag_values() {
        let obj = create_test_dicom_object();

        // 测试单个值的标签
        let instance_numbers: Vec<i32> = get_tag_values(tags::INSTANCE_NUMBER, &obj);
        assert_eq!(instance_numbers, vec![1]);

        // 测试不存在的标签值，应返回空向量
        let series_numbers: Vec<i32> = get_tag_values(tags::SERIES_NUMBER, &obj);
        assert!(series_numbers.is_empty());
    }

    #[test]
    fn test_trim_end_characters() {
        // 创建带有尾部空格和null字符的测试数据
        let obj = InMemDicomObject::from_element_iter([
            DataElement::new(
                tags::PATIENT_NAME,
                VR::PN,
                PrimitiveValue::from("Doe^John \0"),
            ),
        ]);

        let patient_name = get_text_value(&obj, tags::PATIENT_NAME);
        assert_eq!(patient_name, Some("Doe^John".to_string()));
    }
}
