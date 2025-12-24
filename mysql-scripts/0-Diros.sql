CREATE TABLE IF NOT EXISTS dicom_object_meta (
    trace_id            VARCHAR(36)   NOT NULL COMMENT "全局唯一追踪ID，作为主键",
    worker_node_id      VARCHAR(64)   NOT NULL COMMENT "工作节点 ID",
    tenant_id           VARCHAR(64)   NOT NULL COMMENT "租户ID",
    patient_id          VARCHAR(64)   NOT NULL COMMENT "患者ID",
    study_uid           VARCHAR(64)   NULL,
    series_uid          VARCHAR(64)   NULL,
    sop_uid             VARCHAR(64)   NULL,
    file_size           BIGINT        NULL,
    file_path           VARCHAR(1024) NULL,
    transfer_syntax_uid VARCHAR(64)   NULL,
    number_of_frames    INT           NULL,
    created_time        DATETIME      NULL,
    series_uid_hash     BIGINT        NULL,
    study_uid_hash      BIGINT        NULL,
    accession_number    VARCHAR(64)   NULL,
    target_ts           VARCHAR(64)   NULL,
    study_date          DATE          NULL,
    transfer_status     VARCHAR(64)   NULL,
    source_ip           VARCHAR(24)   NULL,
    source_ae           VARCHAR(64)   NULL
)
ENGINE = OLAP
UNIQUE KEY(trace_id)  -- 逻辑主键，自动去重
COMMENT "DICOM 对象元数据表"
DISTRIBUTED BY HASH(trace_id) BUCKETS 10
PROPERTIES (
    "replication_num" = "1",
    "enable_unique_key_merge_on_write" = "true",  -- ⭐ 必须开启（3.x 默认可能已开）
    "light_schema_change" = "true",               -- 允许快速加列
    "store_row_column" = "true"                   -- 加速点查（3.0+ 新特性）
);

-- 创建访问日志表
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
    created_time        DATETIME      NOT NULL COMMENT '创建时间',
    PRIMARY KEY (log_id)
) ENGINE = InnoDB
  COMMENT = 'DICOM访问日志表'
  COLLATE = utf8mb4_unicode_ci;

-- 为访问日志表创建索引
CREATE INDEX idx_access_log_tenant ON dicom_access_log (tenant_id);
CREATE INDEX idx_access_log_user ON dicom_access_log (user_id);
CREATE INDEX idx_access_log_resource ON dicom_access_log (resource_type, resource_id);
CREATE INDEX idx_access_log_time ON dicom_access_log (created_time);
CREATE INDEX idx_access_log_operation ON dicom_access_log (operation_type, operation_result);

#
#
# CREATE ROUTINE LOAD dicom_routine_load ON dicom_object_meta
# COLUMNS (
#     trace_id,
#     worker_node_id = "default_worker",  -- 默认值
#     tenant_id,
#     patient_id,
#     study_uid,
#     series_uid,
#     sop_uid,
#     file_size,
#     file_path,
#     transfer_syntax_uid,
#     number_of_frames,
#     created_time,
#     series_uid_hash,
#     study_uid_hash,
#     accession_number,
#     target_ts,
#     study_date,
#     transfer_status,
#     source_ip,
#     source_ae
# )
# PROPERTIES (
#     "desired_concurrent_number" = "3",
#     "max_batch_interval" = "10",
#     "max_batch_rows" = "300000",
#     "max_batch_size" = "209715200",
#     "format" = "json",
#     "max_error_number" = "1000"
# )
# FROM KAFKA (
#     "kafka_broker_list" = "192.168.1.14:9092",
#     "kafka_topic" = "log_queue",
#     "kafka_partitions" = "0",
#     "property.kafka_default_offsets" = "OFFSET_BEGINNING"
# );





CREATE ROUTINE LOAD dicom_routine_load ON dicom_object_meta
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
    series_uid_hash ,
    study_uid_hash  ,
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
    "jsonpaths" = "[\"$.trace_id\",
                    \"$.worker_node_id\",
                    \"$.tenant_id\",
                    \"$.patient_id\",
                    \"$.study_uid\",
                    \"$.series_uid\",
                    \"$.sop_uid\",
                    \"$.file_size\",
                    \"$.file_path\",
                    \"$.transfer_syntax_uid\",
                    \"$.number_of_frames\",
                    \"$.created_time\",
                    \"$.series_uid_hash\",
                    \"$.study_uid_hash\",
                    \"$.accession_number\",
                    \"$.target_ts\",
                    \"$.study_date\",
                    \"$.transfer_status\",
                    \"$.source_ip\",
                    \"$.source_ae\"]",
    "max_error_number" = "1000"
)
FROM KAFKA (
    "kafka_broker_list" = "192.168.1.14:9092",
    "kafka_topic" = "log_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);