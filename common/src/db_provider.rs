use async_trait::async_trait;
use dicom_object::{DefaultDicomObject, InMemDicomObject};

#[async_trait]
pub trait DbProvider {
    // 保存DICOM信息
    // 返回值：Some(true) 表示成功保存，Some(false) 表示已存在，None 表示保存失败
    async  fn save_dicom_info(&self, tenant_id: &str, dicom_obj: &DefaultDicomObject) -> Option<bool>;
    // 根据DICOM对象的Study Instance UID、Series Instance UID、SOP Instance UID删除DICOM信息
    // 返回值：Some(true) 表示成功删除，Some(false) 表示未删除，None 表示删除失败
    async  fn delete_study_info(&self, tenant_id: &str, study_uid: &str) -> Option<bool>;

    // 根据DICOM对象的Study Instance UID、Series Instance UID删除DICOM信息
    // 返回值：Some(true) 表示成功删除，Some(false) 表示未删除，None 表示删除失败
    async fn delete_series_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Option<bool>;
    // 根据DICOM对象的Study Instance UID、Series Instance UID、SOP Instance UID删除DICOM信息
    // 返回值：Some(true) 表示成功删除，Some(false) 表示未删除，None 表示删除失败
    async  fn delete_instance_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Option<bool>;
}
