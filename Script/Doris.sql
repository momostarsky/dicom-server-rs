-- 存储收图记录  dicom_object_meta
-- 存储切片信息  dicom_image_meta
-- 存储WADO调阅日志 dicom_wado_log
-- 存储访问日志 dicom_access_log
-- DUPLICATE KEY:
-- 不会自动去重，允许多条具有相同 key 的数据存在
-- 所有数据行都会被保留
-- 查询时可能返回多条相同 key 的记录
-- UNIQUE KEY:
-- 自动根据 key 去重
-- 当新数据与已有数据的 key 相同时，会替换旧数据
-- 保证每个 key 只有一条记录
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
    series_uid_hash     VARCHAR(20)   null,
    study_uid_hash      VARCHAR(20)   null,
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

-- 存储访问日志
DROP TABLE IF EXISTS dicom_access_log;
CREATE TABLE IF NOT EXISTS dicom_access_log (
    log_id              VARCHAR(36)   NOT NULL COMMENT '日志ID，作为主键',
    tenant_id           VARCHAR(64)   NOT NULL COMMENT '租户ID',
    user_id             VARCHAR(64)   NOT NULL COMMENT '用户ID',
    username            VARCHAR(128)  NOT NULL COMMENT '用户名',
    operation_type      VARCHAR(32)   NOT NULL COMMENT '操作类型 (READ, WRITE, DELETE, QUERY等)',
    operation_path      VARCHAR(512)  NOT NULL COMMENT '操作路径',
    operation_method    VARCHAR(10)   NOT NULL COMMENT '操作方法 (GET, POST, PUT, DELETE等)',
    operation_result    VARCHAR(16)   NOT NULL COMMENT '操作结果 (SUCCESS, FAILED)',
    resource_type       VARCHAR(32)   NOT NULL COMMENT '资源类型 (STUDY, SERIES, INSTANCE等)',
    resource_id         VARCHAR(64)   NOT NULL COMMENT '资源ID (StudyUID, SeriesUID, SOPInstanceUID)',
    ip_address          VARCHAR(45)   NOT NULL COMMENT 'IP地址',
    user_agent          VARCHAR(512)  NULL COMMENT '用户代理',
    response_time       BIGINT        NOT NULL COMMENT '响应时间(毫秒)',
    description         VARCHAR(1024) NULL COMMENT '操作描述',
    created_time        DATETIME      NOT NULL COMMENT '创建时间'
)
ENGINE=OLAP
DUPLICATE KEY(log_id)  -- 逻辑主键，自动去重
DISTRIBUTED BY HASH(log_id) BUCKETS 1
PROPERTIES("replication_num" = "1");

DROP TABLE IF  EXISTS dicom_state_meta;
CREATE TABLE IF NOT EXISTS dicom_state_meta (
    -- 基本标识信息
                                                tenant_id VARCHAR(64) NOT NULL,
    patient_id VARCHAR(64) NOT NULL,
    study_uid VARCHAR(64) NOT NULL,
    series_uid VARCHAR(64) NOT NULL,
    study_uid_hash  VARCHAR(20)  NOT NULL,
    series_uid_hash  VARCHAR(20)   NOT NULL,
    study_date_origin VARCHAR(8) NOT NULL,

    -- 患者信息
    patient_name VARCHAR(64) NULL,
    patient_sex VARCHAR(1) NULL,
    patient_birth_date DATE NULL,
    patient_birth_time VARCHAR(16) NULL,
    patient_age VARCHAR(16) NULL,
    patient_size DOUBLE NULL,
    patient_weight DOUBLE NULL,
    pregnancy_status INT NULL,

    -- 检查信息
    study_date DATE NOT NULL,
    study_time VARCHAR(16) NULL,
    accession_number VARCHAR(16) NOT NULL,
    study_id VARCHAR(16) NULL,
    study_description VARCHAR(64) NULL,


    -- 序列信息
    modality VARCHAR(16) NULL,
    series_number INT NULL,
    series_date DATE NULL,
    series_time VARCHAR(16) NULL,
    series_description VARCHAR(256) NULL,
    body_part_examined VARCHAR(64) NULL,
    protocol_name VARCHAR(64) NULL,
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
    -- 基本标识信息
                                                tenant_id VARCHAR(64) NOT NULL COMMENT "租户ID",
    patient_id VARCHAR(64) NOT NULL COMMENT "患者ID",
    study_uid VARCHAR(64) NOT NULL COMMENT "检查UID",
    series_uid VARCHAR(64) NOT NULL COMMENT "序列UID",
    sop_uid VARCHAR(64) NOT NULL COMMENT "实例UID",

    -- 哈希值
    study_uid_hash  VARCHAR(20) NOT NULL COMMENT "检查UID哈希值",
    series_uid_hash   VARCHAR(20)   NOT NULL COMMENT "序列UID哈希值",

    -- 时间相关
    study_date_origin DATE NOT NULL COMMENT "检查日期(原始格式)",
    content_date DATE COMMENT "内容日期",
    content_time VARCHAR(32) COMMENT "内容时间",


    -- 图像基本信息
    instance_number INT COMMENT "实例编号",
    image_type VARCHAR(128) COMMENT "图像类型",
    image_orientation_patient VARCHAR(128) COMMENT "图像方向(患者坐标系)",
    image_position_patient VARCHAR(64) COMMENT "图像位置(患者坐标系)",

    -- 图像尺寸参数
    slice_thickness DOUBLE COMMENT "层厚",
    spacing_between_slices DOUBLE COMMENT "层间距",
    slice_location DOUBLE COMMENT "切片位置",

    -- 像素数据属性
    samples_per_pixel INT COMMENT "每个像素采样数",
    photometric_interpretation VARCHAR(32) COMMENT "光度解释",
    width INT COMMENT "图像行数",
    columns INT COMMENT "图像列数",
    bits_allocated INT COMMENT "分配位数",
    bits_stored INT COMMENT "存储位数",
    high_bit INT COMMENT "高比特位",
    pixel_representation INT COMMENT "像素表示法",

    -- 重建参数
    rescale_intercept DOUBLE COMMENT "重建截距",
    rescale_slope DOUBLE COMMENT "重建斜率",
    rescale_type VARCHAR(64) COMMENT "重建类型",
    window_center VARCHAR(64) COMMENT "窗位中心",
    window_width VARCHAR(64) COMMENT "窗宽",

    -- 传输和分类信息
    transfer_syntax_uid VARCHAR(64) NOT NULL COMMENT "传输语法UID",
    pixel_data_location VARCHAR(512) COMMENT "像素数据位置",
    thumbnail_location VARCHAR(512) COMMENT "缩略图位置",
    sop_class_uid VARCHAR(64) NOT NULL COMMENT "SOP类UID",
    image_status VARCHAR(32) COMMENT "图像状态",
    space_size BIGINT COMMENT "占用空间大小",
    created_time DATETIME COMMENT "创建时间",
    updated_time DATETIME COMMENT "更新时间",
    )
ENGINE=OLAP
UNIQUE KEY(tenant_id, patient_id, study_uid, series_uid, sop_uid)
DISTRIBUTED BY HASH(tenant_id) BUCKETS 1
PROPERTIES("replication_num" = "1");

--- 下面的脚步最好逐个执行，避免重复创建---------------------
STOP  ROUTINE LOAD FOR medical_object_load;
STOP  ROUTINE LOAD FOR medical_state_load;
STOP  ROUTINE LOAD FOR medical_image_load;
---------------------------------------------------------
CREATE ROUTINE LOAD medical_object_load ON dicom_object_meta
COLUMNS (
    trace_id,
    worker_node_id,
    tenant_id,
    patient_id,
    study_uid,
    series_uid,
    sop_uid,
    file_size,
    file_path,
    transfer_syntax_uid,
    number_of_frames,
    created_time,
    series_uid_hash,
    study_uid_hash,
    accession_number,
    target_ts,
    study_date,
    transfer_status,
    source_ip,
    source_ae
)
PROPERTIES (
    "desired_concurrent_number" = "3",
    "max_batch_interval" = "10",
    "max_batch_rows" = "300000",
    "max_batch_size" = "209715200",
    "format" = "json",
    "max_error_number" = "1000"
)
FROM KAFKA (
    "kafka_broker_list" = "127.0.0.1:9092",
    "kafka_topic" = "log_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);


CREATE ROUTINE LOAD medical_state_load ON dicom_state_meta
COLUMNS (
        tenant_id  ,
        patient_id ,
        study_uid ,
        series_uid,
        study_uid_hash,
        series_uid_hash,
        study_date_origin,

        -- 患者信息
        patient_name,
        patient_sex ,
        patient_birth_date ,
        patient_birth_time,
        patient_age,
        patient_size,
        patient_weight,
        pregnancy_status,

        -- 检查信息
        study_date,
        study_time,
        accession_number,
        study_id,
        study_description,
        -- 序列信息
        modality,
        series_number,
        series_date,
        series_time,
        series_description,
        body_part_examined,
        protocol_name,
        series_related_instances,
        created_time,
        updated_time = NOW()
)
PROPERTIES (
    "desired_concurrent_number" = "3",
    "max_batch_interval" = "10",
    "max_batch_rows" = "300000",
    "max_batch_size" = "209715200",
    "format" = "json",
    "max_error_number" = "1000",
    "strip_outer_array" = "false"
)
FROM KAFKA (
    "kafka_broker_list" = "127.0.0.1:9092",
    "kafka_topic" = "dicom_state_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);



CREATE ROUTINE LOAD medical_image_load ON dicom_image_meta
COLUMNS (
    tenant_id,
    patient_id,
    study_uid,
    series_uid,
    sop_uid,
    study_uid_hash,
    series_uid_hash,
    study_date_origin,
    content_date,
    content_time,
    instance_number,
    image_type,
    image_orientation_patient,
    image_position_patient,
    slice_thickness,
    spacing_between_slices,
    slice_location,
    samples_per_pixel,
    photometric_interpretation,
    width,
    `columns`,
    bits_allocated,
    bits_stored,
    high_bit,
    pixel_representation,
    rescale_intercept,
    rescale_slope,
    rescale_type,
    window_center,
    window_width,
    transfer_syntax_uid,
    pixel_data_location,
    thumbnail_location,
    sop_class_uid,
    image_status,
    space_size,
    created_time,
    updated_time = NOW()
)
PROPERTIES (
    "desired_concurrent_number" = "3",

    "max_batch_interval" = "10",
    "max_batch_rows" = "300000",
    "max_batch_size" = "209715200",
    "format" = "json",
    "max_error_number" = "1000",
    "strip_outer_array" = "false"

)
FROM KAFKA (
    "kafka_broker_list" = "127.0.0.1:9092",
    "kafka_topic" = "dicom_image_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);

