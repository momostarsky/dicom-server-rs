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
    /// 创建一个新的 `BoundedString`
    ///
    /// 如果输入字符串长度超过指定长度 N，会截断多余的字符
    ///
    /// # 参数
    /// * `s` - 输入的字符串
    ///
    /// # 返回值
    /// 返回一个长度为 N 的 `BoundedString`
    ///
    /// # 注意
    /// 此方法会截断超出指定长度的字符，可能导致数据丢失，请谨慎使用
    pub fn make(s: String) -> BoundedString<N> {
        if s.len() > N {
            Self {
                value: s[..N].to_string(),
            }
        } else {
            Self { value: s }
        }
    }
    /// 创建一个新的 `BoundedString`
    ///
    /// 如果输入字符串长度超过指定长度 N，会截断多余的字符
    ///
    /// # 参数
    /// * `s` - 输入的字符串
    ///
    /// # 返回值
    /// 返回一个长度为 N 的 `BoundedString`
    ///
    /// # 注意
    /// 此方法会截断超出指定长度的字符，可能导致数据丢失，请谨慎使用
    pub fn make_str(s: &str) -> BoundedString<N> {
        if s.len() > N {
            Self {
                value: s[..N].to_string(),
            }
        } else {
            Self {
                value: String::from(s),
            }
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
impl<const N: usize> fmt::Display for BoundedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
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
impl<const N: usize> TryFrom<&String> for BoundedString<N> {
    type Error = BoundedStringError;
    fn try_from(value: &String) -> BoundedResult<Self> {
        BoundedString::from_str(value)
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

    /// 创建一个新的 `FixedLengthString`
    ///
    /// 如果输入字符串长度超过指定长度 N，会截断多余的字符
    ///
    /// # 参数
    /// * `s` - 输入的字符串
    ///
    /// # 返回值
    /// 返回一个长度为 N 的 `FixedLengthString`
    ///
    /// # 注意
    /// 此方法会截断超出指定长度的字符，可能导致数据丢失，请谨慎使用
    pub fn make(s: String) -> FixedLengthString<N> {
        if s.len() > N {
            Self {
                value: s[..N].to_string(),
            }
        } else {
            Self { value: s }
        }
    }
    /// 创建一个新的 `FixedLengthString`
    ///
    /// 如果输入字符串长度超过指定长度 N，会截断多余的字符
    ///
    /// # 参数
    /// * `s` - 输入的字符串
    ///
    /// # 返回值
    /// 返回一个长度为 N 的 `FixedLengthString`
    ///
    /// # 注意
    /// 此方法会截断超出指定长度的字符，可能导致数据丢失，请谨慎使用
    pub fn make_str(s: &str) -> FixedLengthString<N> {
        if s.len() > N {
            Self {
                value: s[..N].to_string(),
            }
        } else {
            Self {
                value: String::from(s),
            }
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
    pub fn make(s: &str) -> Self {
        chrono::NaiveDate::parse_from_str(s, "%Y%m%d").expect("Invalid date format ,expected YYYYMMDD");
        Self {
            value: s.to_string(),
        }
    }
    pub fn as_str(&self) -> &str {
        self.value.as_str()
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

    /// 尝试从字符串值转换为当前类型的安全转换函数
    ///
    /// # 参数
    /// * `value` - 需要转换的字符串值
    ///
    /// # 返回值
    /// * `BoundedResult<Self>` - 转换结果，成功时返回封装后的当前类型实例，失败时返回相应的错误信息
    ///
    /// # 说明
    /// 该函数实现了TryFrom trait，提供了一种安全的字符串到目标类型的转换机制，
    /// 通过返回BoundedResult类型来处理转换过程中可能出现的边界检查和验证错误。
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
