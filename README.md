### 总体架构

1. RedPanda: As a message queue,replace Apache Kafka. For production environments, it is recommended to use RedPanda Enterprise Edition or Apache Kafka.
2. Apache Doris, ClickHouse: As data warehouses. Provide storage for DicomStateMeta, DicomImageMeta, and WadoAccessLog, enabling subsequent querying and statistical analysis.
3. PostgreSQL: As a database. Provides data storage functionality and examination indexing features. Only stores metadata at the PatientInformation, StudyInformation, SeriesInformation levels. Fully utilizes the ACID characteristics of relational databases. For production environments, Citus is recommended.
4. Redis: As a cache. Provides data caching functionality.
5. Nginx: As a reverse proxy server. Provides load balancing, static files, TLS passthrough, etc. For production environments, Nginx Plus or LVS+DR mode is recommended to improve performance.


#### Data Flow

### PACS or Machine

1. Client  CStoreSCU or STOW-RS Send Dicom File To   wado-storescp or WADO-Server Server。

### Server

1. Write Dicom File To Disk -->  PublishMessage To Topic :{ log_quene,storage_queue }
    - ClickHouse or Doris --> Consume Topices: { log_quene } --> Persistent { DicomObjectMeta } To Database

2. Consumer {storage_queue } --> Persistent { DicomStateMeta,DicomImageMeta} To MainDatabase
    - Consumer {dicom_state_queue,dicom_image_queue }
    - --> PublishMessag To Topic :{ dicom_state_queue,dicom_image_queue }

3. ClickHouse or Doris --> Consume Topices: { dicom_state_queue,dicom_image_queue }
    - --> Persistent {DicomStateMeta,DicomImageMeta} To Database

### Mobile or Web

1. WADO-RS or QIDO-RS query study information .
2. Use Cornerstone3D to render image in HTML5 Applications.

### Services Usage and Introduce

| service        | usage                                                                                             | 
|----------------|---------------------------------------------------------------------------------------------------|
| wado-server    | DICOMWEB  WADO-RS RESTFul API ,support oauth2.                                                    | 
| wado-storescp  | CStoreSCP Provider,write dicom file to disk. and publish message to kafka:storage_queue,log_queue |
| wado-consumer  | consumer storage-queue,publish messages to kafka:dicom_state_queue,dicom_image_queue              |
| wado-webworker | generate metadata for wado-server and update related instances for series and study.              |

### how to deploy to test

[Docs/Install](./Docs/Readme.md) 