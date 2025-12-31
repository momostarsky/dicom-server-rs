## Summary

1. RedPanda: Replaces Apache Kafka as a message queue. For production environments, it is recommended to use RedPanda Enterprise Edition or Apache Kafka.
2. Doris or ClickHouse: As data warehouses. Provide storage for DicomStateMeta, DicomImageMeta, and WadoAccessLog, enabling subsequent querying and statistical analysis.
3. PostgreSQL: As a database. Provides data storage functionality and exam indexing features. Only stores metadata at the PatientInformation, StudyInformation, SeriesInformation levels. Fully utilizes the ACID characteristics of relational databases. For production environments, Citus is recommended.
4. Redis: As a cache. Provides data caching functionality.
5. Nginx: As a reverse proxy server. Provides load balancing, static files, TLS passthrough, etc. For production environments, Nginx Plus or LVS+DR mode is recommended to improve performance.

## Data Flow

### PACS or Machine

1. Client (C-Store SCU or STOW-RS) sends DICOM files to wado-storescp or WADO-Server.

### Server

1. Write DICOM File To Disk -->  Publish Message To Topic: {log_queue, storage_queue}
   ---> ClickHouse consumes Topics: {log_queue} --> Persist {DICOMObjectMeta} To ClickHouse Database

2. Consumer {storage_queue} fetches metadata: {DicomStateMeta, DicomImageMeta}
    - --> Persist {DicomStateMeta, DicomImageMeta} To Main Database--PostgreSQL
    - --> Publish Message To Topic: {dicom_state_queue, dicom_image_queue}

3. ClickHouse or Doris --> Consumes Topics: {dicom_state_queue, dicom_image_queue} --> Persist {DicomStateMeta, DicomImageMeta} To Database

### Mobile or Web

1. WADO-RS or QIDO-RS queries study information.
2. Uses Cornerstone3D to render images in HTML5 Applications.

### Services Usage and Introduction

| service        | usage                                                                                                     | 
|----------------|-----------------------------------------------------------------------------------------------------------|
| wado-server    | DICOMWeb WADO-RS RESTful API, supports OAuth2.                                                            | 
| wado-storescp  | C-Store SCP Provider, writes DICOM files to disk and publishes message to Kafka: storage_queue, log_queue |
| wado-consumer  | Consumes storage-queue, publishes messages to Kafka: dicom_state_queue, dicom_image_queue                 |
| wado-webworker | Generates metadata for wado-server and updates related instances for series and study.                    |

### How to deploy for testing

[Docs/Install](./Docs/Readme.md)

### RoadMAP

- [✓] Basic DICOM C-STORE SCP Support
- [✓] Basic DICOMWeb WADO-RS Support
- [✓] Basic DICOM Metadata Extraction and Storage
- [✓] DICOMWeb STOW-RS Support
- [✓] OAuth2 Support  for WADO-RS , STOW-RS
- [ ] Add S3 Storage Support
- [ ] Web-based Viewer Integration
- [ ] Add Prometheus & Grafana Monitoring Support
- [ ] DICOM Query-Retrieve (C-FIND, C-MOVE, C-GET) Support
- [ ] Advanced Metadata Search and Filtering
- [ ] User Management and Access Control
- [ ] Performance Optimization and Scalability Improvements
- [ ] Comprehensive Testing and Documentation