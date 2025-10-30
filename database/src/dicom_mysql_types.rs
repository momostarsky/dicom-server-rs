use crate::dicom_dbtype::{BoundedString, DicomDateString};
use mysql::prelude::*;
use mysql::{FromValueError, Value};
use std::convert::Infallible;

impl<const N: usize> From<BoundedString<N>> for Value {
    fn from(bounded_string: BoundedString<N>) -> Self {
        Value::from(bounded_string.as_str())
    }
}
impl<const N: usize> From<String> for BoundedString<N> {
    fn from(value: String) -> Self {
        BoundedString::<N>::new(value.as_str().parse().unwrap()).unwrap()
    }
}
impl<const N: usize> FromValue for BoundedString<N> {
    type Intermediate = String; // 恢复为 String

    fn from_value(v: Value) -> BoundedString<N> {
        let s = String::from_value(v);
        // 直接使用 try_from 并处理错误
        BoundedString::<N>::try_from(s).unwrap_or_else(|_| BoundedString::<N>::default())
    }
}

impl From<DicomDateString> for Value {
    fn from(dicom_date_string: DicomDateString) -> Self {
        Value::from(dicom_date_string.as_str())
    }
}

impl From<String> for DicomDateString {
    fn from(value: String) -> Self {
        DicomDateString::new(value.as_str())
    }
}
impl FromValue for DicomDateString {
    type Intermediate = String; // 恢复为 String

    fn from_value(v: Value) -> DicomDateString {
        let s = String::from_value(v);
        // 直接使用 try_from 并处理错误
        DicomDateString::try_from(s).unwrap_or_else(|_| DicomDateString::default())
    }
}

// 移除冲突的 From<String> 实现，这些应该在 dicom_dbtype.rs 中已经存在

impl Default for DicomDateString {
    fn default() -> Self {
        DicomDateString {
            value: "00000000".to_string(),
        }
    }
}
