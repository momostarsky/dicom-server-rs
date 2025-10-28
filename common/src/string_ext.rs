use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum BoundedStringError {
    #[snafu(display("String too long: {} > {}", len, max))]
    TooLong { max: usize, len: usize },
    #[snafu(display("String length is: {}  and  expected: {}", len, fixlen))]
    LengthError { fixlen: usize, len: usize },
}

type BoundedResult<T, E = BoundedStringError> = Result<T, E>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
#[derive(Default)]
pub struct BoundedString<const N: usize> {
    value: String,
}

impl<const N: usize> BoundedString<N> {
    pub fn new(s: String) -> BoundedResult<BoundedString<N>> {
        if s.len() > N {
            Err(BoundedStringError::TooLong {
                max: N,
                len: s.len(),
            })
        } else {
            Ok(Self { value: s })
        }
    }
    pub fn new_from_str(s: &str) -> BoundedResult<BoundedString<N>> {
        if s.len() > N {
            Err(BoundedStringError::TooLong {
                max: N,
                len: s.len(),
            })
        } else {
            Ok(Self {
                value: s.to_string(),
            })
        }
    }
    pub fn new_from_string(s: &String) -> BoundedResult<BoundedString<N>> {
        if s.len() > N {
            Err(BoundedStringError::TooLong {
                max: N,
                len: s.len(),
            })
        } else {
            Ok(Self { value: s.clone() })
        }
    }
    pub fn as_str(&self) -> &str {
        &self.value
    }
}
impl<const N: usize> Hash for BoundedString<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<const N: usize> PartialEq for BoundedString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<const N: usize> Eq for BoundedString<N> {}

impl<const N: usize> TryFrom<String> for BoundedString<N> {
    type Error = BoundedStringError;
    fn try_from(s: String) -> BoundedResult<Self> {
        BoundedString::new(s)
    }
}

impl<const N: usize> TryFrom<&str> for BoundedString<N> {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        BoundedString::new_from_str(s)
    }
}

impl<const N: usize> TryFrom<&String> for BoundedString<N> {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        BoundedString::new_from_string(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FixedLengthString<const N: usize> {
    value: String,
}

impl<const N: usize> FixedLengthString<N> {
    pub fn new(s: String) -> BoundedResult<FixedLengthString<N>> {
        if s.len() != N {
            Err(BoundedStringError::LengthError {
                fixlen: N,
                len: s.len(),
            })
        } else {
            Ok(Self { value: s })
        }
    }

    pub fn new_from_str(s: &str) -> BoundedResult<FixedLengthString<N>> {
        if s.len() != N {
            Err(BoundedStringError::LengthError {
                fixlen: N,
                len: s.len(),
            })
        } else {
            Ok(Self {
                value: s.to_string(),
            })
        }
    }

    pub fn new_from_string(s: &String) -> BoundedResult<FixedLengthString<N>> {
        if s.len() != N {
            Err(BoundedStringError::LengthError {
                fixlen: N,
                len: s.len(),
            })
        } else {
            Ok(Self { value: s.clone() })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl<const N: usize> Hash for FixedLengthString<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<const N: usize> PartialEq for FixedLengthString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<const N: usize> Eq for FixedLengthString<N> {}

impl<const N: usize> TryFrom<String> for FixedLengthString<N> {
    type Error = BoundedStringError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        FixedLengthString::new(s)
    }
}

impl<const N: usize> TryFrom<&str> for FixedLengthString<N> {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        FixedLengthString::new_from_str(s)
    }
}

impl<const N: usize> TryFrom<&String> for FixedLengthString<N> {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        FixedLengthString::new_from_string(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SopUidString(BoundedString<64>);

impl SopUidString {
    pub fn from_bounded_string(bounded: BoundedString<64>) -> Self {
        Self(bounded)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl TryFrom<String> for SopUidString {
    type Error = BoundedStringError;
    fn try_from(s: String) -> BoundedResult<Self> {
        BoundedString::new_from_string(&s).map(|bounded| Self(bounded))
    }
}

impl TryFrom<&str> for SopUidString {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        BoundedString::new_from_str(s).map(|bounded| Self(bounded))
    }
}

impl TryFrom<&String> for SopUidString {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        BoundedString::new_from_string(s).map(|bounded| Self(bounded))
    }
}

/// DICOM文件中的表示日期的字符串，格式为 YYYYMMDD, 长度为 8, 例如 "20231005"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct DicomDateString(FixedLengthString<8>);

impl DicomDateString {
    pub fn from_fixed_length_string(fixed: FixedLengthString<8>) -> Self {
        Self(fixed)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl TryFrom<String> for DicomDateString {
    type Error = BoundedStringError;
    fn try_from(s: String) -> BoundedResult<Self> {
        FixedLengthString::new_from_string(&s).map(|fixed| Self(fixed))
    }
}

impl TryFrom<&str> for DicomDateString {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        FixedLengthString::new_from_str(s).map(|fixed| Self(fixed))
    }
}

impl TryFrom<&String> for DicomDateString {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        FixedLengthString::new_from_string(s).map(|fixed| Self(fixed))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct UuidString(FixedLengthString<36>);

impl UuidString {
    pub fn from_fixed_length_string(fixed: FixedLengthString<36>) -> Self {
        Self(fixed)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl TryFrom<String> for UuidString {
    type Error = BoundedStringError;
    fn try_from(s: String) -> BoundedResult<Self> {
        FixedLengthString::new_from_string(&s).map(|fixed| Self(fixed))
    }
}

impl TryFrom<&str> for UuidString {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        FixedLengthString::new_from_str(s).map(|fixed| Self(fixed))
    }
}

impl TryFrom<&String> for UuidString {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        FixedLengthString::new_from_string(s).map(|fixed| Self(fixed))
    }
}

#[derive(Debug, Clone)]
pub struct ExtDicomTimeInvalidError {
    message: String,
}

impl ExtDicomTimeInvalidError {
    pub fn new(message: &str) -> Self {
        ExtDicomTimeInvalidError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ExtDicomTimeInvalidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid DICOM time: {}", self.message)
    }
}

impl std::error::Error for ExtDicomTimeInvalidError {}

/// DICOM时间字符串，格式为 HHMMSS.FFFFFF, 长度为 12, 例如 "123456.123456"
/// 对DICOM. 时间字符串，比如 "123456.123456" 或 "123456"，都可以解析为 ExtDicomTime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ExtDicomTime {
    value: Option<NaiveTime>,
}

impl fmt::Display for ExtDicomTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(time) => write!(f, "{}", time.format("%H:%M:%S%.f")),
            None => write!(f, ""),
        }
    }
}
impl ExtDicomTime {
    pub fn new(p0: Option<NaiveTime>) -> Self {
        Self { value: p0 }
    }

    pub fn as_naive_time(&self) -> Option<&NaiveTime> {
        self.value.as_ref()
    }

    pub fn into_naive_time(self) -> Option<NaiveTime> {
        self.value
    }
    pub fn from_str(s: &str) -> Option<Self> {
        if s.is_empty() {
            return Some(ExtDicomTime::new(None));
        }

        // 处理不同格式的DICOM时间字符串
        let normalized_time = Self::normalize_dicom_time(s)?;
        match NaiveTime::parse_from_str(&normalized_time, "%H%M%S%.f") {
            Ok(t) => Some(ExtDicomTime::new(Some(t))),
            Err(_) => None,
        }
    }

    /// 标准化DICOM时间字符串，确保毫秒部分为6位
    pub fn normalize_dicom_time(s: &str) -> Option<String> {
        if !s.contains('.') {
            // 没有毫秒部分，直接返回
            return Some(s.to_string());
        }

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return None;
        }

        let time_part = parts[0];
        let mut fraction_part = parts[1].to_string();

        // 补齐或截断小数部分到6位
        while fraction_part.len() < 6 {
            fraction_part.push('0');
        }
        fraction_part.truncate(6);

        Some(format!("{}.{}", time_part, fraction_part))
    }
}
impl TryFrom<&str> for ExtDicomTime {
    type Error = ExtDicomTimeInvalidError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ExtDicomTime::from_str(value).ok_or_else(|| ExtDicomTimeInvalidError::new("Invalid format"))
    }
}

impl TryFrom<&String> for ExtDicomTime {
    type Error = ExtDicomTimeInvalidError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        ExtDicomTime::from_str(value).ok_or_else(|| ExtDicomTimeInvalidError::new("Invalid format"))
    }
}

impl TryFrom<String> for ExtDicomTime {
    type Error = ExtDicomTimeInvalidError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ExtDicomTime::from_str(&value)
            .ok_or_else(|| ExtDicomTimeInvalidError::new("Invalid format"))
    }
}

#[derive(Debug, Clone)]
pub struct ExtDicomDateInvalidError {
    message: String,
}

impl ExtDicomDateInvalidError {
    pub fn new(message: &str) -> Self {
        ExtDicomDateInvalidError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ExtDicomDateInvalidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid DICOM date: {}", self.message)
    }
}

impl std::error::Error for ExtDicomDateInvalidError {}

/// DICOM日期字符串，格式为 YYYYMMDD, 长度为 8, 例如 "20231005"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ExtDicomDate {
    value: Option<chrono::NaiveDate>,
}

impl fmt::Display for ExtDicomDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(date) => write!(f, "{}", date.format("%Y%m%d")),
            None => write!(f, ""),
        }
    }
}

impl ExtDicomDate {
    pub fn new(value: Option<chrono::NaiveDate>) -> Self {
        Self { value }
    }

    pub fn as_naive_date(&self) -> Option<&chrono::NaiveDate> {
        self.value.as_ref()
    }

    pub fn into_naive_date(self) -> Option<chrono::NaiveDate> {
        self.value
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if s.is_empty() {
            return Some(ExtDicomDate::new(None));
        }

        match chrono::NaiveDate::parse_from_str(s, "%Y%m%d") {
            Ok(date) => Some(ExtDicomDate::new(Some(date))),
            Err(_) => None,
        }
    }
}

impl TryFrom<&str> for ExtDicomDate {
    type Error = ExtDicomDateInvalidError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ExtDicomDate::from_str(value).ok_or_else(|| ExtDicomDateInvalidError::new("Invalid format"))
    }
}

impl TryFrom<&String> for ExtDicomDate {
    type Error = ExtDicomDateInvalidError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        ExtDicomDate::from_str(value).ok_or_else(|| ExtDicomDateInvalidError::new("Invalid format"))
    }
}

impl TryFrom<String> for ExtDicomDate {
    type Error = ExtDicomDateInvalidError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ExtDicomDate::from_str(&value)
            .ok_or_else(|| ExtDicomDateInvalidError::new("Invalid format"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    use dicom_encoding::snafu::ResultExt;
    use snafu::Whatever;
    #[test]
    fn test_dicom_date_from_str() {
        // 测试有效的日期格式
        let date = ExtDicomDate::from_str("20231005");
        assert!(date.is_some());
        let date = date.unwrap();
        assert!(date.value.is_some());

        // 测试空字符串
        let date = ExtDicomDate::from_str("");
        assert!(date.is_some());
        let date = date.unwrap();
        assert!(date.value.is_none());
    }

    #[test]
    fn test_dicom_date_invalid_format() {
        // 测试无效的日期格式
        let date = ExtDicomDate::from_str("2023-10-05"); // 不符合DICOM格式
        assert!(date.is_none());

        let date = ExtDicomDate::from_str("231005"); // 缺少年份
        assert!(date.is_none());
    }

    #[test]
    fn test_dicom_date_try_from_str() {
        // 测试 TryFrom<&str>
        let date_result: Result<ExtDicomDate, ExtDicomDateInvalidError> = "20231005".try_into();
        assert!(date_result.is_ok());

        // 测试无效格式
        let date_result: Result<ExtDicomDate, ExtDicomDateInvalidError> = "invalid".try_into();
        assert!(date_result.is_err());
    }

    #[test]
    fn test_dicom_date_try_from_string() {
        // 测试 TryFrom<String>
        let date_result: Result<ExtDicomDate, ExtDicomDateInvalidError> = "20231005".to_string().try_into();
        assert!(date_result.is_ok());

        // 测试无效格式
        let date_result: Result<ExtDicomDate, ExtDicomDateInvalidError> = "invalid".to_string().try_into();
        assert!(date_result.is_err());
    }

    #[test]
    fn test_dicom_date_display() {
        // 测试有值的日期显示
        let dicom_date = ExtDicomDate::from_str("20231005").unwrap();
        let display_str = format!("{}", dicom_date);
        assert_eq!(display_str, "20231005");

        // 测试无值的日期显示
        let dicom_date_none = ExtDicomDate::new(None);
        let display_str_none = format!("{}", dicom_date_none);
        assert_eq!(display_str_none, "");
    }

    #[test]
    fn test_dicom_date_accessors() {
        // 测试访问器方法
        let dicom_date = ExtDicomDate::from_str("20231005").unwrap();
        let naive_date = dicom_date.as_naive_date();
        assert!(naive_date.is_some());

        let dicom_date_none = ExtDicomDate::new(None);
        let naive_date_none = dicom_date_none.as_naive_date();
        assert!(naive_date_none.is_none());
    }
    #[test]
    fn test_bounded_string_valid_length() {
        // 测试正常长度的字符串
        let s = "hello".to_string();
        let bounded: BoundedString<10> = BoundedString::new(s).unwrap();
        assert_eq!(bounded.as_str(), "hello");
    }

    #[test]
    fn test_bounded_string_exact_length() {
        // 测试正好达到最大长度的字符串
        let s = "1234567890".to_string();
        let bounded: BoundedString<10> = BoundedString::new(s).unwrap();
        assert_eq!(bounded.as_str(), "1234567890");
    }

    #[test]
    fn test_bounded_string_from_string() {
        // 测试 TryFrom<String> 实现
        let s = "hello".to_string();
        let bounded: BoundedString<10> = s.try_into().unwrap();
        assert_eq!(bounded.as_str(), "hello");
    }

    #[test]
    fn test_bounded_string_from_str() {
        // 测试 TryFrom<&str> 实现
        let bounded: BoundedString<10> = "hello".try_into().unwrap();
        assert_eq!(bounded.as_str(), "hello");
    }

    #[test]
    fn test_bounded_string_from_string_ref() {
        // 测试 TryFrom<&String> 实现
        let s = "hello".to_string();
        let bounded: BoundedString<10> = (&s).try_into().unwrap();
        assert_eq!(bounded.as_str(), "hello");
    }

    #[test]
    fn test_bounded_string_equality() {
        // 测试相等性
        let s1 = "hello".to_string();
        let s2 = "hello".to_string();
        let bounded1: BoundedString<10> = BoundedString::new(s1).unwrap();
        let bounded2: BoundedString<10> = BoundedString::new(s2).unwrap();
        assert_eq!(bounded1, bounded2);
    }

    #[test]
    fn test_bounded_string_hash() {
        // 测试哈希一致性
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let s = "hello".to_string();
        let bounded: BoundedString<10> = BoundedString::new(s).unwrap();

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        bounded.hash(&mut hasher1);
        bounded.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    #[should_panic]
    fn test_bounded_string_too_long() {
        // 测试超长字符串应该panic（因为使用了unwrap）
        let s = "this string is definitely too long for the limit".to_string();
        let _bounded: BoundedString<10> = BoundedString::new(s).unwrap();
    }

    #[test]
    fn test_bounded_string_error_handling() {
        // 测试错误处理
        let s = "this string is definitely too long for the limit".to_string();
        let result: Result<BoundedString<10>, _> = BoundedString::new(s);
        assert!(result.is_err());
    }
    #[test]

    fn test_bounded_string_watch_context_handling() {
        let s = "this string is definitely too long for the limit".to_string();
        let result: Result<BoundedString<10>, _> = BoundedString::new(s.clone())
            .with_whatever_context(|err| format!("Failed to create BoundedString: {}", err));

        assert!(result.is_err());
        let err: Whatever = result.unwrap_err();
        println!("Serialized JSON: {}", err.to_string());
        assert_eq!(
            err.to_string(),
            format!(
                "Failed to create BoundedString: String too long: {} > 10",
                s.len()
            )
        );
    }

    #[test]

    fn test_bounded_string_watch_context_handling2() {
        let s = "this string is definitely too long for the limit".to_string();
        let result: Result<BoundedString<10>, BoundedStringError> = BoundedString::new(s.clone());

        assert!(result.is_err());
        let err: BoundedStringError = result.unwrap_err();
        println!("Serialized JSON: {}", err.to_string());
        assert_eq!(
            err.to_string(),
            BoundedStringError::TooLong {
                max: 10,
                len: s.len()
            }
            .to_string()
        );
    }
    #[test]
    fn test_dicom_store_meta_json_fmt() {
        use crate::dicom_object_meta::DicomStoreMeta;
        let meta = DicomStoreMeta {
            trace_id: "0199e6ae-8148-7e73-8d6c-c435bf126fe4".try_into().unwrap(),
            worker_node_id: "DICOM_STORE_SCP".try_into().unwrap(),
            tenant_id: "1234567890".try_into().unwrap(),
            patient_id: "10535086".try_into().unwrap(),
            study_uid: "1.3.12.2.1107.5.2.12.21149.2021013010174414769824".try_into().unwrap(),
            series_uid: "1.3.46.670589.26.902153.2.20210130.102145.856875".try_into().unwrap(),
            sop_uid: "1.3.46.670589.26.902153.4.20210130.102215.856875.0".try_into().unwrap(),
            file_size: 11910596,
            file_path: "/media/dhz/DCP/dcm/1234567890/20210130/1.3.12.2.1107.5.2.12.21149.2021013010174414769824/1.3.46.670589.26.902153.2.20210130.102145.856875/1.3.46.670589.26.902153.4.20210130.102215.856875.0.dcm".try_into().unwrap(),
            transfer_syntax_uid: "1.2.840.10008.1.2.1".try_into().unwrap(),
            number_of_frames: 1,
            created_time: DateTime::from_timestamp(1728971020, 104453242)
                .unwrap()
                .naive_utc(),
            series_uid_hash: BoundedString::<20>::new("102145856875".to_string()).unwrap(),
            study_uid_hash: BoundedString::<20>::new("2021013010174".to_string()).unwrap(),
            accession_number: "14769824".try_into().unwrap(),
            target_ts: "1.2.840.10008.1.2.1".try_into().unwrap(),
            study_date: "20210130".try_into().unwrap(),
            transfer_status: crate::dicom_object_meta::TransferStatus::NoNeedTransfer,
            source_ip: "127.0.0.1".try_into().unwrap(),
            source_ae: "STORE-SCU".try_into().unwrap(),
        };

        let json = serde_json::to_string(&meta).unwrap();
        // 验证序列化结果不包含 "value" 字符串
        assert!(!json.contains("\"value\""));
        // 验证包含关键字段值
        assert!(json.contains("\"trace_id\":\"0199e6ae-8148-7e73-8d6c-c435bf126fe4\""));
        assert!(json.contains("\"worker_node_id\":\"DICOM_STORE_SCP\""));
        assert!(json.contains("\"tenant_id\":\"1234567890\""));

        println!("Serialized JSON: {}", json);
    }
}
