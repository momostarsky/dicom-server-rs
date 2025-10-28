use serde::{Deserialize, Serialize};

pub mod cert_helper;
pub mod change_file_transfer;
pub mod cornerstonejs;
pub mod database_entities;
pub mod database_factory;
pub mod database_provider;
pub mod database_provider_base;
pub mod dicom_json_helper;
pub mod dicom_object_meta;
pub mod dicom_utils;
pub mod license_manager;
pub mod message_sender;
pub mod message_sender_kafka;
pub mod mysql_provider;
pub mod server_config;
pub mod string_ext;
pub mod uid_hash;
pub mod utils;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DicomMessage {
    pub tenant: String,
    pub transfer_syntax: String,
    pub sop_instance_uid: String,
    pub study_instance_uid: String,
    pub series_instance_uid: String,
    pub patient_id: String,
    pub file_size: u64,
    pub file_path: String,
    pub accession_number: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let msg = DicomMessage {
            tenant: "tenant1".to_string(),
            transfer_syntax: "1.2.840.10008.1.2".to_string(),
            sop_instance_uid: "1.2.3".to_string(),
            study_instance_uid: "1.2.3".to_string(),
            series_instance_uid: "1.2.3".to_string(),
            patient_id: "123".to_string(),
            file_size: 1024,
            file_path: "/tmp/123.dcm".to_string(),
            accession_number: "123333".to_string(),
        };

        assert_eq!(msg.tenant, "tenant1");
        assert_eq!(msg.transfer_syntax, "1.2.840.10008.1.2");
        assert_eq!(msg.study_instance_uid, "1.2.3");
        assert_eq!(msg.series_instance_uid, "1.2.3");
        assert_eq!(msg.patient_id, "123");

        assert_eq!(msg.file_size, 1024);
        assert_eq!(msg.file_path, "/tmp/123.dcm");
        assert_eq!(msg.accession_number, "123333");
    }
}
