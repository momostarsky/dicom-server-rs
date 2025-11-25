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
