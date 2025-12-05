use crate::dicom_dbtype::{BoundedString, DicomDateString, FixedLengthString};
use postgres_types::private::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type};
use std::error::Error;

impl<const N: usize> ToSql for BoundedString<N> {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        <&str as ToSql>::to_sql(&self.as_str(), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&str as ToSql>::to_sql_checked(&self.as_str(), ty, out)
    }
}
// impl From<String> for DicomDateString {
//     fn from(value: String) -> Self {
//         DicomDateString::from_db(value.as_str())
//     }
// }
//
// // 移除冲突的 From<String> 实现，这些应该在 dicom_dbtype.rs 中已经存在
//
// impl Default for DicomDateString {
//     fn default() -> Self {
//         DicomDateString {
//             value: "00000000".to_string(),
//         }
//     }
// }

impl<const N: usize> FromSql<'_> for BoundedString<N> {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let str_val = <&str as FromSql>::from_sql(ty, raw)?;
        Ok(BoundedString::try_from(str_val.to_string())
            .map_err(|e| format!("Failed to create BoundedString FromSql: {}", e))?)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}
impl ToSql for DicomDateString {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        <&str as ToSql>::to_sql(&self.as_str(), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&str as ToSql>::to_sql_checked(&self.as_str(), ty, out)
    }
}
impl FromSql<'_> for DicomDateString {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let str_val = <&str as FromSql>::from_sql(ty, raw)?;
        Ok(DicomDateString::try_from(str_val.to_string())
            .map_err(|e| format!("Failed to create DicomDateString FromSql: {}", e))?)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}




impl<const N: usize> ToSql for FixedLengthString<N> {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        // 将 FixedLengthString 转换为 &str，然后使用标准的 ToSql 实现
        <&str as ToSql>::to_sql(&self.as_str(), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        // 接受文本类型
        <&str as ToSql>::accepts(ty)
    }

    fn to_sql_checked(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}
