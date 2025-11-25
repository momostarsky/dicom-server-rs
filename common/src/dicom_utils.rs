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
    match dicom_obj.element(tag) {
        Ok(e) => match e.to_date() {
            Ok(date) => {
                let year: i32 = *date.year() as i32;
                let month = if let Some(month) = date.month() {
                    *month as u32
                } else {
                    1
                };
                let day = if let Some(day) = date.day() {
                    *day as u32
                } else {
                    1
                };
                NaiveDate::from_ymd_opt(year, month, day)
            }
            Err(_) => None,
        },
        Err(_) => None,
    }
}

/// 解析DICOM时间字符串，支持多种格式：
/// - %H%M%S.%f (带毫秒)
/// - %H%M%S. (带点但无毫秒)
/// - %H%M%S (不带毫秒)
// fn parse_dicom_time(time_str: &str) -> Result<NaiveTime, chrono::ParseError> {
//     // 尝试解析带毫秒的格式 (%H%M%S.%f)
//     NaiveTime::parse_from_str(time_str, "%H%M%S.%f")
//         .or_else(|_| {
//             // 尝试解析带点但无毫秒的格式 (%H%M%S.)
//             NaiveTime::parse_from_str(time_str, "%H%M%S.")
//         })
//         .or_else(|_| {
//             // 尝试解析不带毫秒的格式 (%H%M%S)
//             NaiveTime::parse_from_str(time_str, "%H%M%S")
//         })
// }
pub fn get_time_value_dicom(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<NaiveTime> {
    match dicom_obj.element(tag) {
        Ok(e) => match e.to_time() {
            Ok(date) => {
                let year = *date.hour() as u32;
                let month = *date.minute().unwrap() as u32;
                let day = *date.second().unwrap() as u32;
                let mrs = date.millisecond().unwrap();
                NaiveTime::from_hms_micro_opt(year, month, day, mrs)
            }
            Err(_) => None,
        },
        Err(_) => None,
    }

    // get_text_value(dicom_obj, tag).and_then(|s| {
    //     // 简单处理时间格式，实际可能需要更复杂的解析
    //     if !s.is_empty() {
    //         match parse_dicom_time(&s[..]) {
    //             Ok(date) => Some(date),
    //             Err(_) => None,
    //         }
    //     } else {
    //         None
    //     }
    // })
}

pub fn get_datetime_value_dicom(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<NaiveDateTime> {
    match dicom_obj.element(tag) {
        Ok(e) => match e.to_datetime() {
            Ok(datetime) => {
                let date = datetime.date();
                let time = datetime.time();
                let year = *date.year() as i32;
                let month = if let Some(month) = date.month() {
                    *month as u32
                } else {
                    1
                };

                let day = if let Some(day) = date.day() {
                    *day as u32
                } else {
                    1
                };

                let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                if let Some(time) = time {
                    let hour = *time.hour() as u32;
                    let minute = if let Some(minute) = time.minute() {
                        *minute as u32
                    } else {
                        0
                    };

                    let second = if let Some(second) = time.second() {
                        *second as u32
                    } else {
                        0
                    };

                    let microsecond = if let Some(microsecond) = time.millisecond() {
                        microsecond
                    } else {
                        0
                    };
                    let t =
                        NaiveTime::from_hms_milli_opt(hour, minute, second, microsecond).unwrap();
                    Option::from(NaiveDateTime::new(d, t))
                } else {
                    let t = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
                    Option::from(NaiveDateTime::new(d, t))
                }
            }
            Err(_) => None,
        },
        Err(_) => None,
    }

    // get_text_value(dicom_obj, tag).and_then(|s| {
    //     // 简单处理时间格式，实际可能需要更复杂的解析
    //
    //     if !s.is_empty() {
    //         match NaiveDateTime::parse_from_str(&s[..], "%Y%m%d%H%M%S%.f") {
    //             Ok(date) => Some(date),
    //             Err(_) => None,
    //         }
    //     } else {
    //         None
    //     }
    // })
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
