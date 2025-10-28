use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::hash::{Hash, Hasher};

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
    pub value: String,
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

    // 使用 deref 方式访问
    pub fn as_ref(&self) -> &String {
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
    pub(crate) value: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct UidHashString(pub(crate) BoundedString<20>);

impl UidHashString {
    pub fn make_from(uid: &str) -> Self {
        let mut hasher = SeaHasher::new();
        uid.hash(&mut hasher);
        let hash_value = hasher.finish();
        // 格式化为20位字符串，前面补X
        let hash_str = format!("{:020X}", hash_value);
        Self(BoundedString::new_from_str(&hash_str).unwrap())
    }
    pub fn from_bounded_string(bounded: BoundedString<20>) -> Self {
        Self(bounded)
    }
    pub fn from_string(s: String) -> Self {
        Self(BoundedString::new_from_string(&s).unwrap())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    // 使用 deref 方式访问
    pub fn as_ref(&self) -> &String {
        self.0.as_ref()
    }
}

impl TryFrom<&String> for UidHashString {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        BoundedString::new_from_string(s).map(|fixed| Self(fixed))
    }
}
// 为 UidHashString 实现 From<String>
impl From<String> for UidHashString {
    fn from(s: String) -> Self {
        // 假设 UidHashString 是基于 BoundedString 或类似包装类型
        // 根据实际的 UidHashString 定义调整实现
        UidHashString::try_from(s).unwrap_or_else(|_| {
            // 或者提供一个默认值或处理错误的方式
            panic!("Failed to convert String to UidHashString")
        })
    }
}

// 同时实现 From<&str>
impl From<&str> for UidHashString {
    fn from(s: &str) -> Self {
        UidHashString::try_from(s)
            .unwrap_or_else(|_| panic!("Failed to convert &str to UidHashString"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDate};

    use dicom_encoding::snafu::ResultExt;
    use snafu::Whatever;

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

    use std::fmt;

    // 为 BoundedString 实现 Display trait
    impl<const N: usize> fmt::Display for BoundedString<N> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    // 为 FixedLengthString 实现 Display trait
    impl<const N: usize> fmt::Display for FixedLengthString<N> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    // 为 SopUidString 实现 Display trait
    impl fmt::Display for SopUidString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0.as_str())
        }
    }

    // 为 DicomDateString 实现 Display trait
    impl fmt::Display for DicomDateString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0.as_str())
        }
    }

    // 为 UuidString 实现 Display trait
    impl fmt::Display for UuidString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0.as_str())
        }
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
            series_uid_hash: UidHashString::make_from("123456789"),
            study_uid_hash: UidHashString::make_from("323456789"),
            accession_number: "14769824".try_into().unwrap(),
            target_ts: "1.2.840.10008.1.2.1".try_into().unwrap(),
            study_date: NaiveDate::from_ymd_opt(2021, 1, 30).unwrap(),
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

