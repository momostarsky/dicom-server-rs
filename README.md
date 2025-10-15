### 总体架构

1. Apache-kafka 作为消息队列.
2. Apache-Doris 作为数据仓库.提供数据分析功能.
3. MySQL8 作为数据库.提供数据存储功能.及检查索引功能.
4. Redis 作为缓存.提供数据缓存功能.
5. Nginx 作为反向代理服务器.提供负载均衡和静态文件服务.

收图文件服务接收的文件,先存储到本地,再通过 kafka 发送到消息队列.
消息体包括以下内容:

{
TransferSynatx, SopInstancheUID, StudyInstanceUID,SeriesInstanceUID, PatientID, FileName, FileSize, FilePath
}
消息分发到多个队列:
1. 存储队列: 存储文件信息,文件存储路径,文件大小.
2. 索引队列: 提取文件TAG信息, 包括PatientInfomation, StudyInformation, SeriesInformation, ImageInformation.并写入Doris库
3. 转换队列: 对于部分传输语法,因为Cornerstone3D无法解析,需要转换成CornerstoneJS能够解析的格式.转换失败的写入Doris转换记录表.
### 需要安装 dicom-org-cn.pem 文件到证书目录.
```bash
curl https://dicom.org.cn:8443/ca  >>  ~/dicom-org-cn.crt
sudo cp ~/dicom-org-cn.crt  /usr/local/share/ca-certificates/dicom-org-cn.crt  
sudo update-ca-certificates
```

### 创建任务 Apache Doris 从 Kafka 加载数据
```MySQL

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
```

```load from kafka
STOP ROUTINE LOAD FOR dicom_routine_load;
-- 重新创建任务，指定 JSON 格式
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

 
SHOW ROUTINE LOAD FOR dicom_routine_load;

```