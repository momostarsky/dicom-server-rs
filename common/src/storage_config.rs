use database::dicom_meta::DicomStateMeta;
use crate::server_config::load_config;

/// 生成 UID 的哈希值, 对于不足20位时，定长设为20位,前置补0
pub fn hash_uid(uid: &str) -> String {
    use seahash::SeaHasher;
    use std::hash::Hasher;

    let mut hasher = SeaHasher::new();
    hasher.write(uid.as_bytes());
    let hash_value = hasher.finish();

    // 将 u64 转换为字符串，并用前导零填充到 20 位
    format!("{:020}", hash_value)
}

pub fn dicom_study_dir(
    study_info: &DicomStateMeta,
    create_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let dicom_store_path = &app_config.local_storage.dicm_store_path;
    let study_dir = format!(
        "{}/{}/{}/{}",
        dicom_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str(),
        study_info.study_uid.as_str()
    );
    if create_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("create dicom_study_dir failed: {} with:'{}'", study_dir, e),
            )
        })?;
    }
    Ok(study_dir)
}

pub fn dicom_series_dir(
    study_info: &DicomStateMeta,
    create_when_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let dicom_store_path = &app_config.local_storage.dicm_store_path;
    let study_dir = format!(
        "{}/{}/{}/{}/{}",
        dicom_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str(),
        study_info.study_uid.as_str(),
        study_info.series_uid.as_str()
    );
    if create_when_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("create dicom_series_dir failed: {} with:'{}'", study_dir, e),
            )
        })?;
    }
    Ok(study_dir)
}

pub fn make_series_dicom_dir(
    tenant_id: &str,
    study_date: &str,
    study_uid: &str,
    series_uid: &str,
    create_when_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let dicom_store_path = &app_config.local_storage.dicm_store_path;
    let study_dir = format!(
        "{}/{}/{}/{}/{}",
        dicom_store_path, tenant_id, study_date, study_uid, series_uid
    );
    if create_when_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "create make_series_dicom_dir failed: {} with:'{}'",
                    study_dir, e
                ),
            )
        })?;
    }
    Ok(study_dir)
}

pub fn json_metadata_for_study(
    study_info: &DicomStateMeta,
    create_when_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let json_store_path = &app_config.local_storage.json_store_path;
    let study_dir = format!(
        "{}/{}/metadata/{}",
        json_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str()
    );
    let json_path = format!("{}/{}.json", study_dir, study_info.study_uid.as_str());
    if create_when_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory:  '{}': {}", study_dir, e),
            )
        })?
    }
    Ok(json_path)
}

pub fn json_metadata_for_series(
    study_info: &DicomStateMeta,
    create_when_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let json_store_path = &app_config.local_storage.json_store_path;
    let study_dir = format!(
        "{}/{}/metadata/{}/{}",
        json_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str(),
        study_info.study_uid.as_str()
    );
    let json_path = format!("{}/{}.json", study_dir, study_info.series_uid.as_str());
    if create_when_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory:  '{}': {}", study_dir, e),
            )
        })?
    }
    Ok(json_path)
}

pub fn dicom_file_path(dir: &str, sop_uid: &str) -> String {
    format!("{}/{}.dcm", dir, sop_uid)
}
