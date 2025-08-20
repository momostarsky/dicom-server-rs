use crate::database_entities::{ImageEntity, PatientEntity, SeriesEntity, StudyEntity};
use async_trait::async_trait;
use dicom_object::DefaultDicomObject;

#[async_trait]
pub trait DbProvider :Send + Sync  {
    // 保存DICOM信息
    // 返回值：Some(true) 表示成功保存，Some(false) 表示已存在，None 表示保存失败
    async fn save_dicom_info(
        &self,
        tenant_id: &str,
        dicom_obj: &DefaultDicomObject,
    ) -> Option<bool>;
    async fn save_patient_info(
        &self,
        tenant_id: &str,
        patient_lists: &[PatientEntity],
    ) -> Option<bool>;
    async fn save_study_info(&self, tenant_id: &str, study_lists: &[StudyEntity]) -> Option<bool>;
    async fn save_series_info(
        &self,
        tenant_id: &str,
        series_lists: &[SeriesEntity],
    ) -> Option<bool>;

    async fn save_instance_info(&self, tenant_id: &str, dicom_obj: &[ImageEntity]) -> Option<bool>;
    // 根据DICOM对象的Study Instance UID、Series Instance UID、SOP Instance UID删除DICOM信息
    // 返回值：Some(true) 表示成功删除，Some(false) 表示未删除，None 表示删除失败
    async fn delete_study_info(&self, tenant_id: &str, study_uid: &str) -> Option<bool>;

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
    async fn delete_instance_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Option<bool>;

    // 判断DICOM对象是否存在
    // 返回值：Some(true) 表示存在，Some(false) 表示不存在，None 表示查询失败
    async fn patient_exists(&self, tenant_id: &str, patient_id: &str) -> Option<bool>;
    // 判断DICOM对象是否存在
    // 参数：
    //   - tenant_id: 租户ID
    //   - patient_id: 患者ID
    //   - study_uid: Study Instance UID
    // 返回值：Some(true) 表示存在，Some(false) 表示不存在，None 表示查询失败
    async fn patient_study_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
    ) -> Option<bool>;
    // 判断DICOM对象是否存在
    // 参数：
    //   - tenant_id: 租户ID
    //   - patient_id: 患者ID
    //   - study_uid: Study Instance UID
    //   - series_uid: Series Instance UID
    // 返回值：Some(true) 表示存在，Some(false) 表示不存在，None 表示查询失败
    async fn patient_series_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Option<bool>;
    // 判断DICOM对象是否存在
    // 参数：
    //   - tenant_id: 租户ID
    //   - patient_id: 患者ID
    //   - study_uid: Study Instance UID
    //   - instance_uid: SOP Instance UID
    // 返回值：Some(true) 表示存在，Some(false) 表示不存在，None 表示查询失败
    async fn patient_instance_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Option<bool>;

    async fn persist_to_database(
        &self,
        tenant_id: &str,
        patient_list: &[PatientEntity],
        study_list: &[StudyEntity],
        series_list: &[SeriesEntity],
        images_list: &[ImageEntity],
    ) -> Option<bool>;

    // 获取患者信息
    async fn get_study_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Option<StudyEntity>;
}
