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

impl<const N: usize> fmt::Display for BoundedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
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
    pub fn from_str(s: &str) -> BoundedResult<BoundedString<N>> {
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
    pub fn from_string(s: &String) -> BoundedResult<BoundedString<N>> {
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



impl<const N: usize> TryFrom<&str> for BoundedString<N> {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        BoundedString::from_str(s)
    }
}
impl<const N: usize> TryFrom<String> for BoundedString<N> {
    type Error = BoundedStringError;

    fn try_from(value: String) -> BoundedResult<Self> {
        BoundedString::new(value)
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FixedLengthString<const N: usize> {
    pub(crate) value: String,
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

/// DICOM文件中的表示日期的字符串，格式为 YYYYMMDD, 长度为 8, 例如 "20231005"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct DicomDateString {
    pub(crate) value: String,
}

impl DicomDateString {
    pub fn new(s: String) -> BoundedResult<DicomDateString> {
        // 使用 NaiveDate 验证日期格式和有效性
        chrono::NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(|_| {
            BoundedStringError::LengthError {
                fixlen: 8,
                len: s.len(),
            }
        })?;
        Ok(Self { value: s })
    }
    pub fn from_str(s: &str) -> BoundedResult<DicomDateString> {
        // 使用 NaiveDate 验证日期格式和有效性
        chrono::NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(|_| {
            BoundedStringError::LengthError {
                fixlen: 8,
                len: s.len(),
            }
        })?;
        Ok(Self {
            value: s.to_string(),
        })
    }
}

// 为 DicomDateString 实现 Display trait
impl fmt::Display for DicomDateString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<&str> for DicomDateString {
    type Error = BoundedStringError;
    fn try_from(s: &str) -> BoundedResult<Self> {
        // 使用 NaiveDate 验证日期格式和有效性
        chrono::NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(|_| {
            BoundedStringError::LengthError {
                fixlen: 8,
                len: s.len(),
            }
        })?;
        Ok(Self {
            value: s.to_string(),
        })
    }
}

impl TryFrom<&String> for DicomDateString {
    type Error = BoundedStringError;
    fn try_from(s: &String) -> BoundedResult<Self> {
        // 使用 NaiveDate 验证日期格式和有效性
        chrono::NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(|_| {
            BoundedStringError::LengthError {
                fixlen: 8,
                len: s.len(),
            }
        })?;
        Ok(Self { value: s.clone() })
    }
}

impl TryFrom<String> for DicomDateString {
    type Error = BoundedStringError;

    fn try_from(value: String) -> BoundedResult<Self> {
        // 使用 NaiveDate 验证日期格式和有效性
        chrono::NaiveDate::parse_from_str(value.as_str(), "%Y%m%d").map_err(|_| {
            BoundedStringError::LengthError {
                fixlen: 8,
                len: value.len(),
            }
        })?;
        Ok(Self {
            value: value.clone(),
        })
    }
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

    pub fn from_str(s: &str) -> BoundedResult<FixedLengthString<N>> {
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

    pub fn from_string(s: &String) -> BoundedResult<FixedLengthString<N>> {
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

impl DicomDateString {
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    #[allow(dead_code)]
    pub(crate) fn from_db(s: &str) -> Self {
        // 使用 NaiveDate 验证日期格式和有效性
        chrono::NaiveDate::parse_from_str(&s, "%Y%m%d")
            .map_err(|_| BoundedStringError::LengthError {
                fixlen: 8,
                len: s.len(),
            })
            .expect(
                format!(
                    "DicomDateString::make_from_db  only support YYYYMMDD format, but got {}",
                    s
                )
                .as_str(),
            );

        Self {
            value: s.to_string(),
        }
    }
}
