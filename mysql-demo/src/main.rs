use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{ MySql, MySqlPool, Transaction};
use tracing::{error, info};

// static DB_URL: &str = " mysql://dicomstore:hzjp%23123@192.168.1.14:3306/dicomdb";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientEntity {
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub patient_sex: Option<String>,
    pub patient_birth_datetime: Option<NaiveDateTime>,
}
// study.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntity {
    pub study_instance_uid: String,
    pub patient_id: String,
    pub medical_alerts: Option<String>,
    pub allergies: Option<String>,
    pub pregnancy_status: Option<i32>,
    pub occupation: Option<String>,
    pub additional_patient_history: Option<String>,
}
pub struct MySqlDatabase {
    pool: MySqlPool,
}

impl MySqlDatabase {
    pub fn new(pool: MySqlPool) -> Self {
        info!("MySqlProvider created with pool: {:?}", pool);

        Self { pool }
    }

    pub async fn save_person_info(
        &self,
        person_info: &[PatientEntity],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error> {
        for person in person_info {
            let query = r#"
            INSERT INTO patient_info (patient_id, patient_name, patient_sex, patient_birth_datetime)
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE patient_name = VALUES(patient_name), patient_sex = VALUES(patient_sex), patient_birth_datetime = VALUES(patient_birth_datetime)
        "#;
            sqlx::query(query)
                .bind(&person.patient_id)
                .bind(&person.patient_name)
                .bind(&person.patient_sex)
                .bind(&person.patient_birth_datetime)
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }
    pub async fn save_study_info(
        &self,
        studies: &[StudyEntity],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error> {
        for study in studies {
            let query = r#"
            INSERT INTO study_info (study_instance_uid, patient_id, medical_alerts, allergies, pregnancy_status, occupation, additional_patient_history)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE medical_alerts = VALUES(medical_alerts), allergies = VALUES(allergies),
            pregnancy_status = VALUES(pregnancy_status), occupation = VALUES(occupation),
            additional_patient_history = VALUES(additional_patient_history)
        "#;

            sqlx::query(query)
                .bind(&study.study_instance_uid)
                .bind(&study.patient_id)
                .bind(&study.medical_alerts)
                .bind(&study.allergies)
                .bind(&study.pregnancy_status)
                .bind(&study.occupation)
                .bind(&study.additional_patient_history)
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }

    pub async fn save_batch_with_tx(&self, patients: &[PatientEntity], studies: &[StudyEntity]) {
        let mut tx = match self.pool.begin().await {
            Ok(tx) => tx,
            Err(_) => {
                panic!("Failed to begin transaction");
            }
        };

        // 保存患者信息
        if let Err(e) = self.save_person_info(patients, &mut tx).await {
            error!("Failed to save person info: {:?}", e);
            panic!("Failed to save person info: {}", e);
        }

        // 保存检查信息
        if let Err(e) = self.save_study_info(studies, &mut tx).await {
            error!("Failed to save study info: {:?}", e);
            panic!("Failed to save study info: {}", e);
        }
        match tx.commit().await {
            Ok(_) => {
                info!("Transaction committed successfully");
            }
            Err(e) => {
                info!("Transaction commit failed: {:?}", e);
                panic!("Failed to commit transaction");
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
