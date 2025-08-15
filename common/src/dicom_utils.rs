use dicom_core::Tag;
use dicom_object::InMemDicomObject;

pub fn get_text_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
    dicom_obj
        .element(tag)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim_end_matches('\0').to_string())
}

pub fn get_date_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
    get_text_value(dicom_obj, tag).and_then(|s| {
        // 尝试解析DICOM日期格式 (YYYYMMDD)
        if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            Some(format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]))
        } else {
            None
        }
    })
}

pub fn get_time_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
    get_text_value(dicom_obj, tag).and_then(|s| {
        // 简单处理时间格式，实际可能需要更复杂的解析
        if !s.is_empty() { Some(s) } else { None }
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
