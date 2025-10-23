CREATE TABLE dicom_state_meta (
-- 基本标识信息
tenant_id   VARCHAR(64) NOT NULL,
patient_id  VARCHAR(64) NOT NULL,
study_uid   VARCHAR(64)  NOT NULL,
series_uid  VARCHAR(64)  NOT NULL,
study_uid_hash   BIGINT NOT NULL,
series_uid_hash  BIGINT NOT NULL,
-- 患者相关信息
patient_name VARCHAR(64),
patient_sex CHAR(1),
patient_birth_date DATE,
patient_birth_time TIME,
patient_age VARCHAR(16),
patient_size DOUBLE PRECISION,
patient_weight DOUBLE PRECISION,
pregnancy_status INTEGER,
-- 检查相关信息
study_date DATE NOT NULL,
study_date_origin  varchar(8) NOT NULL,
study_time TIME,
accession_number VARCHAR(16) NOT NULL,
study_id VARCHAR(16),
study_description VARCHAR(64),
-- 序列相关信息
modality VARCHAR(16),
series_number INTEGER,
series_date DATE,
series_time TIME,
series_description VARCHAR(256),
body_part_examined VARCHAR(64),
protocol_name VARCHAR(64),
series_related_instances INTEGER,
-- 时间戳
created_time TIMESTAMP(6),
updated_time TIMESTAMP(6),
-- 主键约束
PRIMARY KEY (tenant_id, study_uid, series_uid)
);
-- 同一个StudyUID 只能有一个AccessionNumber
create unique index index_state_unique on dicom_state_meta(tenant_id, study_uid, accession_number);