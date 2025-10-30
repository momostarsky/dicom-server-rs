use crate::dicom_dbtype::{BoundedString, DicomDateString};
use chrono::NaiveDate;
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
