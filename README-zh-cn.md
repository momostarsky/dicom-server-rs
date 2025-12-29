# DICOM 服务器架构文档

## 总体架构

1. **RedPanda**: 作为 Apache Kafka 的替代消息队列。生产环境建议使用 RedPanda 企业版或 Apache Kafka。

2. **Doris or ClickHouse**: 作为数据仓库，提供 `DicomStateMeta`、`DicomImageMeta` 及 `WadoAccessLog` 存储，支持后续的查询及统计分析。

3. **PostgreSQL**: 作为数据库，提供数据存储功能及检查索引功能，仅存储 `PatientInformation`、`StudyInformation`、`SeriesInformation` 这一级别的元数据，充分利用关系数据库的 ACID 特性。生产环境建议采用 Citus。

4. **Redis**: 作为缓存，提供数据缓存功能。

5. **Nginx**: 作为反向代理服务器，提供负载均衡、静态文件、TLS 透传等功能。生产环境建议采用 Nginx Plus 或 LVS+DR 模式以提升性能。

## 数据流程

### 数据采集终端

客户端通过 `CStoreSCU` 或 `STOW-RS` 协议发送 DICOM 文件到 `wado-storescp`  或 `WADO-Server` 服务。

### 服务器处理流程

1. **文件存储与初步处理**
   - 写入 DICOM 文件到磁盘
   - 发布消息到队列: `{ log_queue, storage_queue }`
   - ClickHouse 或 Doris 数据库消费消息队列 `{ log_queue }`，持久化 `DicomObjectMeta`

2. **主索引信息处理**
   - wado-consumer  服务读取消息队列 `{ storage_queue }`
   - 持久化主索引信息 `{ DicomStateMeta, DicomImageMeta }`
   - 发布消息到 Topic `{ dicom_state_queue, dicom_image_queue }`

3. **索引信息完善**
   - ClickHouse 或 Doris 数据库消费消息队列 `{ dicom_state_queue, dicom_image_queue }`
   - 持久化索引信息 `{ DicomStateMeta, DicomImageMeta }`

### 用户端应用

1. 通过 `WADO-RS` 或 `QIDO-RS` 查询检查列表
2. 采用 Cornerstone3D 库呈现检查图像

## 服务用途说明

| 服务名称           | 用途说明                                                                |
|----------------|---------------------------------------------------------------------|
| wado-server    | 提供 DICOMWEB 的 `WADO-RS`、`STOW-RS` RESTful API，支持 OAuth2 认证          |
| wado-storescp  | `CStoreSCP` 提供者，负责写入 DICOM 文件到磁盘，并发布消息到 `storage_queue`、`log_queue` |
| wado-consumer  | 消费队列 `storage-queue` 并发布消息到 `dicom_state_queue`、`dicom_image_queue` |
| wado-webworker | 生成元数据以优化 wado-server 性能，并更新 series 和 study 的切片数                     |