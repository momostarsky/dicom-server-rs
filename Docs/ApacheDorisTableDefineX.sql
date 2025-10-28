drop  table IF   EXISTS  dicom_object_meta;
create table IF NOT EXISTS  dicom_object_meta
(
    tenant_id           varchar(64)   not null comment '租户ID',
    patient_id          varchar(64)   not null comment '患者ID',
    study_uid           varchar(64)   null,
    series_uid          varchar(64)   null,
    sop_uid             varchar(64)   null,
    file_size           bigint        null,
    file_path           varchar(512) null,
    transfer_syntax_uid varchar(64)   null,
    number_of_frames    int           null,
    created_time        datetime      null,
    series_uid_hash     decimal(32,0)  null,
    study_uid_hash      decimal(32,0)  null,
    accession_number    varchar(64)   null,
    target_ts           varchar(64)   null,
    study_date          date          null,
    transfer_status     varchar(64)   null,
    source_ip           varchar(24)   null,
    source_ae           varchar(64)   null,
    trace_id            varchar(36)   not null comment '全局唯一追踪ID，作为主键',
    worker_node_id      varchar(64)   not null comment '工作节点 ID'
)
ENGINE=OLAP
DUPLICATE KEY(tenant_id,patient_id,study_uid,series_uid,sop_uid)  -- 逻辑主键，自动去重
DISTRIBUTED BY HASH(tenant_id) BUCKETS 1
PROPERTIES("replication_num" = "1");


DROP TABLE IF  EXISTS dicom_state_meta;
CREATE TABLE IF NOT EXISTS dicom_state_meta (
  -- 基本标识信息
    tenant_id VARCHAR(64) NOT NULL,
    patient_id VARCHAR(64) NOT NULL,
    study_uid VARCHAR(64) NOT NULL,
    series_uid VARCHAR(64) NOT NULL,
    study_uid_hash DECIMAL(32,0) NOT NULL,
    series_uid_hash DECIMAL(32,0)  NOT NULL,
    study_date_origin VARCHAR(8) NOT NULL,

    -- 患者信息
    patient_name VARCHAR(64) NULL,
    patient_sex VARCHAR(1) NULL,
    patient_birth_date DATE NULL,
    patient_birth_time VARCHAR(16) NULL,
    patient_age VARCHAR(16) NULL,
    patient_size DOUBLE NULL,
    patient_weight DOUBLE NULL,
    medical_alerts VARCHAR(64) NULL,
    allergies VARCHAR(64) NULL,
    pregnancy_status INT NULL,
    occupation VARCHAR(64) NULL,

    -- 检查信息
    study_date DATE NOT NULL,
    study_time VARCHAR(16) NULL,
    accession_number VARCHAR(16) NOT NULL,
    study_id VARCHAR(16) NULL,
    study_description VARCHAR(64) NULL,
    referring_name VARCHAR(64) NULL,
    admission_id VARCHAR(64) NULL,
    performing_name VARCHAR(64) NULL,

    -- 序列信息
    modality VARCHAR(16) NULL,
    series_number INT NULL,
    series_date DATE NULL,
    series_time VARCHAR(16) NULL,
    series_description VARCHAR(256) NULL,
    body_part_examined VARCHAR(64) NULL,
    protocol_name VARCHAR(64) NULL,
    operators_name VARCHAR(64) NULL,
    manufacturer VARCHAR(64) NULL,
    institution_name VARCHAR(64) NULL,
    device_serial_number VARCHAR(64) NULL,
    software_versions VARCHAR(64) NULL,
    series_related_instances INT NULL,

    -- 时间戳
    created_time DATETIME NULL,
    updated_time DATETIME NULL
)
ENGINE=OLAP
UNIQUE KEY(tenant_id, patient_id, study_uid, series_uid)
DISTRIBUTED BY HASH(tenant_id) BUCKETS 1
PROPERTIES("replication_num" = "1");


DROP TABLE IF   EXISTS dicom_image_meta ;
CREATE TABLE IF NOT EXISTS dicom_image_meta (
  tenant_id VARCHAR(64) COMMENT "租户ID",
  patient_id VARCHAR(64) COMMENT "患者ID",
  study_uid VARCHAR(64) COMMENT "检查UID",
  series_uid VARCHAR(64) COMMENT "序列UID",
  sop_uid VARCHAR(64) COMMENT "实例UID",
  study_uid_hash DECIMAL(32,0) COMMENT "检查UID哈希值",
  series_uid_hash  DECIMAL(32,0)  COMMENT "序列UID哈希值",
  study_date_origin DATE COMMENT "检查日期(原始格式)",
  instance_number INT COMMENT "实例号",
  content_date DATE COMMENT "内容日期",
  content_time VARCHAR(32) COMMENT "内容时间",
  image_type VARCHAR(128) COMMENT "图像类型",
  image_orientation_patient VARCHAR(128) COMMENT "图像方向患者信息",
  image_position_patient VARCHAR(64) COMMENT "图像位置患者信息",
  slice_thickness DOUBLE COMMENT "层厚",
  spacing_between_slices DOUBLE COMMENT "层间距离",
  slice_location DOUBLE COMMENT "层面位置",
  samples_per_pixel INT COMMENT "每像素采样数",
  photometric_interpretation VARCHAR(32) COMMENT "光学解释",
  width INT COMMENT "图像行数",
  columns INT COMMENT "图像列数",
  bits_allocated INT COMMENT "分配的位数",
  bits_stored INT COMMENT "存储的位数",
  high_bit INT COMMENT "高比特位",
  pixel_representation INT COMMENT "像素表示法",
  rescale_intercept DOUBLE COMMENT "重缩放截距",
  rescale_slope DOUBLE COMMENT "重缩放斜率",
  rescale_type VARCHAR(64) COMMENT "重缩放类型",
  window_center VARCHAR(64) COMMENT "窗位中心",
  window_width VARCHAR(64) COMMENT "窗宽",
  transfer_syntax_uid VARCHAR(64) COMMENT "传输语法UID",
  pixel_data_location VARCHAR(512) COMMENT "像素数据位置",
  thumbnail_location VARCHAR(512) COMMENT "缩略图位置",
  sop_class_uid VARCHAR(64) COMMENT "SOP类UID",
  image_status VARCHAR(32) COMMENT "图像状态",
  space_size BIGINT COMMENT "空间大小",
  created_time DATETIME COMMENT "创建时间",
  updated_time DATETIME COMMENT "更新时间"
)
ENGINE=OLAP
UNIQUE KEY(tenant_id, patient_id, study_uid, series_uid, sop_uid)
DISTRIBUTED BY HASH(tenant_id) BUCKETS 1
PROPERTIES("replication_num" = "1");