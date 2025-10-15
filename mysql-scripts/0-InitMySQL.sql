
DROP TABLE IF  EXISTS ImageEntity;

DROP TABLE IF EXISTS SeriesEntity;

DROP TABLE IF EXISTS StudyEntity;

DROP TABLE IF EXISTS PatientEntity;


create table ImageEntity
(
    tenant_id                              varchar(64)                              not null comment '租户ID',
    SOPInstanceUID                         varchar(64)                              not null comment 'SOP实例UID (0008,0018)',
    SeriesInstanceUID                      varchar(64)                              not null comment '序列实例UID (外键，关联 SeriesEntity) (0020,000E)',
    StudyInstanceUID                       varchar(64)                              not null comment '检查实例UID (冗余但常用，便于查询) (0020,000D)',
    PatientID                              varchar(64)                              not null comment '患者ID (冗余但常用) (0010,0020)',
    InstanceNumber                         int                                      null comment '实例编号 (0020,0013) - IS, VM=1, max 12 chars → INT',
    ImageComments                          text                                     null comment '图像注释 (0020,4000) - LT, VM=1, max 10240 chars',
    ContentDate                            date                                     null comment '内容日期 (0008,0023) - DA',
    ContentTime                            time(6)                                  null comment '内容时间 (0008,0033) - TM',
    AcquisitionDate                        date                                     null comment '内容日期 (0008,0022) - DA',
    AcquisitionTime                        time(6)                                  null comment '内容时间 (0008,0032) - TM',
    AcquisitionDateTime                    datetime(6)                              null comment '采集日期时间 (0008,002A) - DT, 合并 DA+TM',
    ImageType                              varchar(64)                              null comment '图像类型 (0008,0008) - CS, VM=1-n, e.g., ORIGINAL/PRIMARY/M/A',
    ImageOrientationPatient                varchar(128)                             null comment '图像方向(患者) (0020,0037) - DS, VM=6, e.g., "1   1 "',
    ImagePositionPatient                   varchar(128)                             null comment '图像位置(患者) (0020,0032) - DS, VM=3, e.g., "-128.0-128.098.0"',
    SliceThickness                         decimal(8, 4)                            null comment '层厚 (mm) (0018,0050) - DS, VM=1',
    SpacingBetweenSlices                   decimal(8, 4)                            null comment '层间距 (mm) (0018,0088) - DS, VM=1',
    SliceLocation                          decimal(8, 4)                            null comment '切片位置 (0020,1041) - DS, VM=1',
    SamplesPerPixel                        tinyint                                  null comment '每像素样本数 (0028,0002) - US, VM=1, e.g., 1=灰度, 3=彩色',
    PhotometricInterpretation              varchar(16)                              null comment '光度学解释 (0028,0004) - CS, VM=1, e.g., MONOCHROME2, RGB',
    Width                                  smallint                                 null comment '行数 (0028,0010) - US, VM=1, max 65535',
    Columns                                smallint                                 null comment '列数 (0028,0011) - US, VM=1, max 65535',
    BitsAllocated                          tinyint                                  null comment '分配位数 (0028,0100) - US, VM=1, e.g., 8, 16',
    BitsStored                             tinyint                                  null comment '存储位数 (0028,0101) - US, VM=1',
    HighBit                                tinyint                                  null comment '最高有效位 (0028,0102) - US, VM=1',
    PixelRepresentation                    tinyint                                  null comment '像素表示 (0028,0103) - US, VM=1, 0=无符号, 1=有符号',
    RescaleIntercept                       decimal(8, 4)                            null comment '重缩放截距 (0028,1052) - DS, VM=1',
    RescaleSlope                           decimal(8, 4)                            null comment '重缩放斜率 (0028,1053) - DS, VM=1',
    RescaleType                            varchar(16)                              null comment '重缩放类型 (0028,1054) - LO, VM=1, e.g., "HU" for CT',
    NumberOfFrames                         int                                      null comment '帧数 (0028,0008) - IS, VM=1, max 8 chars → INT',
    AcquisitionDeviceProcessingDescription varchar(128)                              null comment '采集设备处理描述 (0018,1010) - LO, VM=1',
    AcquisitionDeviceProcessingCode        varchar(256)                              null comment '采集设备处理代码 (0018,1012) - SH, VM=1',
    DeviceSerialNumber                     varchar(16)                              null comment '设备序列号 (0018,1000) - LO, VM=1, max 64 → 取16',
    SoftwareVersions                       varchar(64)                              null comment '软件版本 (0018,1020) - LO, VM=1-n → 取64',
    TransferSyntaxUID                      varchar(64)                              null comment '传输语法UID (0002,0010) - UI, e.g., 压缩或未压缩格式',
    PixelDataLocation                      varchar(255)                             null comment '像素数据存储路径/URL (非DICOM Tag，业务扩展)',
    ThumbnailLocation                      varchar(255)                             null comment '缩略图路径 (业务扩展)',
    SOPClassUID                            varchar(64)                              not null comment 'SOP类UID (0008,0016) - UI, e.g., CT Image Storage',
    ImageStatus                            varchar(16) default 'ACTIVE'             null comment '图像状态 (业务扩展: ACTIVE/ARCHIVED/DELETED)',
    SpaceSize                              bigint      default 0                    null comment '占用空间大小 (字节，业务扩展)',
    CreatedTime                            datetime(6) default CURRENT_TIMESTAMP(6) null,
    UpdatedTime                            datetime(6) default CURRENT_TIMESTAMP(6) null on update CURRENT_TIMESTAMP(6),
    WindowWidth                            varchar(64)                              null comment '窗宽 (0028,1051) - LO, VM=1, e.g., "HU" for CT',
    WindowCenter                           varchar(64)                              null comment '窗位 (0028,1050) - LO, VM=1, e.g., "HU" for CT',
    primary key (tenant_id, SOPInstanceUID)
)
    comment '图像实体表，存储单帧DICOM图像实例' collate = utf8mb4_unicode_ci;

create index idx_content_date
    on ImageEntity (ContentDate);

create index idx_instance_number
    on ImageEntity (InstanceNumber);

create index idx_patient_id
    on ImageEntity (PatientID);

create index idx_series_uid
    on ImageEntity (SeriesInstanceUID);

create index idx_sop_class_uid
    on ImageEntity (SOPClassUID);

create index idx_study_uid
    on ImageEntity (StudyInstanceUID);

create table PatientEntity
(
    tenant_id        varchar(64)                              not null comment '租户ID',
    PatientID        varchar(64)                              not null comment '患者ID (0010,0020)',
    PatientName      varchar(192)                             null comment '患者姓名 (0010,0010)',
    PatientBirthDate date                                     null comment '患者出生日期 (0010,0030)',
    PatientSex       char                                     null comment '患者性别 (0010,0040)',
    PatientBirthTime time                                     null comment '患者出生时间 (0010,0032)',
    EthnicGroup      varchar(16)                              null comment '民族 (0010,2160)',
    CreatedTime      datetime(6) default CURRENT_TIMESTAMP(6) null,
    UpdatedTime      datetime(6) default CURRENT_TIMESTAMP(6) null on update CURRENT_TIMESTAMP(6),
    primary key (tenant_id, PatientID)
)   ENGINE = InnoDB CHARSET = utf8mb4
    comment '患者实体表，符合DICOM标准，支持多租户' collate = utf8mb4_unicode_ci;

create index idx_patient_id
    on PatientEntity (PatientID);

create table SeriesEntity
(
    tenant_id                      varchar(64)                              not null comment '租户ID',
    SeriesInstanceUID              varchar(64)                              not null comment '序列实例UID (0020,000E)',
    StudyInstanceUID               varchar(64)                              not null comment '检查实例UID (外键) (0020,000D)',
    PatientID                      varchar(64)                              not null comment '患者ID (外键，关联 PatientEntity) (0010,0020)',
    Modality                       varchar(16)                              not null comment '模态 (0008,0060) - CS, VM=1, max 16 chars (如 CT, MR, XR)',
    SeriesNumber                   int                                      null comment '序列号 (0020,0011) - IS, VM=1, max 12 chars → 用 INT',
    SeriesDate                     date                                     null comment '序列日期 (0008,0021) - DA',
    SeriesTime                     time(6)                                  null comment '序列时间 (0008,0031) - TM',
    SeriesDescription              varchar(64)                              null comment '序列描述 (0008,103E) - LO, VM=1, max 64 chars',
    BodyPartExamined               varchar(64)                              null comment '检查部位 (0018,0015) - CS, VM=1, max 16 chars',
    ProtocolName                   varchar(64)                              null comment '协议名称 (0018,1030) - LO, VM=1, max 64 chars',
    AcquisitionNumber              int                                      null comment '采集号 (0020,0012) - IS, VM=1, max 12 chars → INT',
    AcquisitionTime                time(6)                                  null comment '内容时间 (0008,0032) - TM',
    AcquisitionDate                date                                     null comment '采集日期 (0008,0022) - DA',
    PerformingPhysicianName        varchar(192)                             null comment '执行医生姓名 (0008,1050) - PN, 可继承自 Study',
    OperatorsName                  varchar(192)                             null comment '操作员姓名 (0008,1070) - PN, VM=1-n',
    NumberOfSeriesRelatedInstances int                                      null comment '该序列关联的图像数量 (0020,1209) - IS',
    ReceivedInstances              int         default 0                    null comment '已接收实例数 (业务扩展)',
    SpaceSize                      bigint      default 0                    null comment '占用空间大小 (字节，业务扩展)',
    CreatedTime                    datetime(6) default CURRENT_TIMESTAMP(6) null,
    UpdatedTime                    datetime(6) default CURRENT_TIMESTAMP(6) null on update CURRENT_TIMESTAMP(6),
    AcquisitionDateTime            datetime(6)                              null comment '采集日期时间 (0008,002A) - DT, 合并 DA+TM',
    primary key (tenant_id, SeriesInstanceUID)
)   ENGINE = InnoDB CHARSET = utf8mb4
    comment '序列实体表' collate = utf8mb4_unicode_ci;

create index idx_body_part
    on SeriesEntity (BodyPartExamined);

create index idx_modality
    on SeriesEntity (Modality);

create index idx_series_number
    on SeriesEntity (SeriesNumber);

create index idx_study_uid
    on SeriesEntity (StudyInstanceUID);

create table StudyEntity
(
    tenant_id                varchar(64)                              not null comment '租户ID',
    StudyInstanceUID         varchar(64)                              not null comment '检查实例UID (0020,000D)',
    PatientID                varchar(64)                              not null comment '患者ID (外键，关联 PatientEntity) (0010,0020)',
    PatientAge               varchar(10)                              null comment '患者年龄 (0010,1010)',
    PatientSize              decimal(5, 2)                            null comment '患者身高 (m)',
    PatientWeight            decimal(5, 2)                            null comment '患者体重 (kg)',
    MedicalAlerts            varchar(1024)                            null comment '医疗警报 (0010,2000)',
    Allergies                varchar(1024)                            null comment '过敏史 (0010,2110)',
    PregnancyStatus          smallint                                 null comment '妊娠状态 (0010,21C0)',
    Occupation               varchar(32)                              null comment '职业 (0010,2180)',
    AdditionalPatientHistory text                                     null comment '附加患者历史 (0010,21B0)',
    PatientComments          text                                     null comment '患者注释 (0010,4000)',
    StudyDate                date                                     null comment '检查日期 (0008,0020) - DA',
    StudyTime                time                                     null comment '检查时间 (0008,0030) - TM',
    AccessionNumber          varchar(16)                              null comment '检查号 (0008,0050) - SH, VM=1, max 16 chars',
    StudyID                  varchar(16)                              null comment '检查ID (0020,0010) - SH, VM=1, max 16 chars',
    StudyDescription         varchar(64)                              null comment '检查描述 (0008,1030) - LO, VM=1, max 64 chars',
    ReferringPhysicianName   varchar(192)                             null comment '转诊医生姓名 (0008,0090) - PN, VM=1, max 64×3',
    AdmissionID              varchar(16)                              null comment '住院号 (0038,0010) - LO, VM=1, max 64 chars → 取16',
    PerformingPhysicianName  varchar(192)                             null comment '执行医生姓名 (0008,1050) - PN, VM=1-n',
    ProcedureCodeSequence    text                                     null comment '检查过程代码序列 (0008,1032) - SQ, 复杂结构，暂存为JSON或文本',
    ReceivedInstances        int         default 0                    null comment '接收实例数量 (业务扩展)',
    SpaceSize                bigint      default 0                    null comment '占用空间大小 (字节，业务扩展)',
    CreatedTime              datetime(6) default CURRENT_TIMESTAMP(6) null,
    UpdatedTime              datetime(6) default CURRENT_TIMESTAMP(6) null on update CURRENT_TIMESTAMP(6),
    primary key (tenant_id, StudyInstanceUID)
)
    ENGINE = InnoDB CHARSET = utf8mb4
    comment '检查实体表' collate = utf8mb4_unicode_ci;

create index idx_accession_number
    on StudyEntity (AccessionNumber);

create index idx_patient_id
    on StudyEntity (PatientID);

create index idx_study_date
    on StudyEntity (StudyDate);

create index idx_study_id
    on StudyEntity (StudyID);

create table dicom_object_meta
(
    id                  bigint auto_increment
        primary key,
    tenant_id           varchar(64)                              not null,
    patient_id          varchar(64)                              not null,
    study_uid           varchar(64)                              not null,
    series_uid          varchar(64)                              not null,
    sop_uid             varchar(64)                              not null,
    file_size           bigint                                   not null,
    file_path           varchar(512)                             not null,
    transfer_syntax_uid varchar(64)                              not null,
    number_of_frames    int                                      not null,
    created_at          timestamp   default CURRENT_TIMESTAMP    null,
    updated_at          timestamp   default CURRENT_TIMESTAMP    null on update CURRENT_TIMESTAMP,
    CreatedTime         datetime(6) default CURRENT_TIMESTAMP(6) null,
    UpdatedTime         datetime(6) default CURRENT_TIMESTAMP(6) null on update CURRENT_TIMESTAMP(6),
    constraint unique_sop_instance
        unique (tenant_id, sop_uid)
)   ENGINE = InnoDB CHARSET = utf8mb4
    collate = utf8mb4_unicode_ci;

create index idx_patient_id
    on dicom_object_meta (patient_id);

create index idx_series_uid
    on dicom_object_meta (series_uid);

create index idx_sop_uid
    on dicom_object_meta (sop_uid);

create index idx_study_uid
    on dicom_object_meta (study_uid);

create index idx_tenant_id
    on dicom_object_meta (tenant_id);

