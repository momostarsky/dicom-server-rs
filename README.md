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

### 服务用途及说明
|服务| 用途                                                    |  
|---|-------------------------------------------------------|
|wado-server| DICOMWEB  WADO-RS RESTFul API 支持 OAuth2 认证.           | 
|wado-storescp| 只负责写磁盘和消息队列                                       |
|wado-consumer| 从storage-queue读,往dicom_state_queue,dicom_image_queue写 |
|wado-webworker| storage-queue                                         |


###  wado-server

    DICOM Web WADO-RS API 接口实现 ,根据配置选项决定是否开启 OAuth2 认证.

### wado-storescp

    DICOM CStoreSCP 服务,接收DICOM文件并写入磁盘,同是分发消息到Kafka:  storage_queue,log_queue      
    - 1.存储文件路径: ${ROOT}/<TenantID>/<StudyInstanceUID>/<SeriesInstanceUID>/<SopInstanceUID>.dcm
    - 2.往消息队列: storage_queue ,log_queue 发送消息

### wado-consumer

    消费Kafka消息队列storage_queue ,提取DicomStateMeta 到主数据库,并通过消息队列:dicom_state_queue,dicom_image_queue发布DicomStateMeta,DicomImageMeta 到Doris数据库
    - 1.从storage-queue读,往dicom_state_queue,dicom_image_queue写
    - 2.提取DicomStateMeta 到主数据库,
    - 3.通过消息队列:dicom_state_queue,dicom_image_queue发布DicomStateMeta,DicomImageMeta 到Doris数据库 Stream_Load 模式
### wado-webworker

    定期扫描数据库,根据最后更新时间生成JSON格式的metadata用于加速WADO-Server的访问
