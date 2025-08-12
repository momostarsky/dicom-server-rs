CREATE TABLE PatientEntity
(
    tenant_id                VARCHAR(64) NOT NULL COMMENT '租户ID',
    PatientID                VARCHAR(64) NOT NULL COMMENT '患者ID (0010,0020)',
    -- 基本信息
    PatientName              VARCHAR(192) COMMENT '患者姓名 (0010,0010)',
    PatientBirthDate         DATE COMMENT '患者出生日期 (0010,0030)',
    PatientSex               CHAR(1) COMMENT '患者性别 (0010,0040)',
    PatientBirthTime         TIME COMMENT '患者出生时间 (0010,0032)',
    EthnicGroup              VARCHAR(16) COMMENT '民族 (0010,2160)',
    -- 时间戳
    CreatedTime              DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '记录创建时间',
    UpdatedTime              DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '记录更新时间',
    -- ✅ 设置联合主键
    PRIMARY KEY (tenant_id, PatientID),
    -- ✅ 为单独查询 PatientID 时提供高效索引（可选）
    KEY                      idx_patient_id (PatientID)
) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci
  COMMENT='患者实体表，符合DICOM标准，支持多租户';

CREATE TABLE StudyEntity
(
    tenant_id               VARCHAR(64) NOT NULL COMMENT '租户ID',
    -- 主键：检查唯一标识符
    StudyInstanceUID        VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '检查实例UID (0020,000D)',
    -- 外键：关联患者
    PatientID               VARCHAR(64) NOT NULL COMMENT '患者ID (外键，关联 PatientEntity) (0010,0020)',
    CONSTRAINT fk_study_patient FOREIGN KEY (PatientID) REFERENCES PatientEntity (PatientID) ON DELETE CASCADE,
    -- 检查时的病人基本 信息
    PatientAge       VARCHAR(10) COMMENT '患者年龄 (0010,1010)',
    PatientSize      DECIMAL(5, 2) COMMENT '患者身高 (m)',
    PatientWeight    DECIMAL(5, 2) COMMENT '患者体重 (kg)',
    MedicalAlerts    VARCHAR(1024) COMMENT '医疗警报 (0010,2000)',
    Allergies        VARCHAR(1024) COMMENT '过敏史 (0010,2110)',
    PregnancyStatus  SMALLINT COMMENT '妊娠状态 (0010,21C0)',
    Occupation             VARCHAR(32) COMMENT '职业 (0010,2180)',
    AdditionalPatientHistory TEXT COMMENT '附加患者历史 (0010,21B0)',
    PatientComments          TEXT COMMENT '患者注释 (0010,4000)',
    -- 检查基本信息
    StudyDate               DATE COMMENT '检查日期 (0008,0020) - DA',
    StudyTime               TIME COMMENT '检查时间 (0008,0030) - TM',
    AccessionNumber         VARCHAR(16) COMMENT '检查号 (0008,0050) - SH, VM=1, max 16 chars',
    StudyID                 VARCHAR(16) COMMENT '检查ID (0020,0010) - SH, VM=1, max 16 chars',
    StudyDescription        VARCHAR(64) COMMENT '检查描述 (0008,1030) - LO, VM=1, max 64 chars',
    -- 检查类型与目的
    StudyStatusID           VARCHAR(16) COMMENT '检查状态 (0032,1030) - CS, VM=1, max 16 chars',
    StudyPriorityID         VARCHAR(10) COMMENT '检查优先级 (0032,1031) - CS, VM=1, max 10 chars',
    ReferringPhysicianName  VARCHAR(192) COMMENT '转诊医生姓名 (0008,0090) - PN, VM=1, max 64×3',
    AdmissionID             VARCHAR(16) COMMENT '住院号 (0038,0010) - LO, VM=1, max 64 chars → 取16',
    PatientAgeAtStudy       VARCHAR(10) COMMENT '检查时患者年龄 (0010,1010) - AS, 来自图像或计算',
    -- 其他信息
    PerformingPhysicianName VARCHAR(192) COMMENT '执行医生姓名 (0008,1050) - PN, VM=1-n',
    ProcedureCodeSequence   TEXT COMMENT '检查过程代码序列 (0008,1032) - SQ, 复杂结构，暂存为JSON或文本',
    StudyComments           TEXT COMMENT '检查注释 (0032,4000) - LT, VM=1, max 10240 chars',
    -- 时间戳
    CreatedTime             DATETIME DEFAULT CURRENT_TIMESTAMP,
    UpdatedTime             DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    -- 索引
    INDEX                   idx_patient_id (PatientID),
    INDEX                   idx_accession_number (AccessionNumber),
    INDEX                   idx_study_date (StudyDate),
    INDEX                   idx_study_id (StudyID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='检查实体表';


CREATE TABLE SeriesEntity
(
    tenant_id                      VARCHAR(64) NOT NULL COMMENT '租户ID',
    -- 主键：序列唯一标识符
    SeriesInstanceUID              VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '序列实例UID (0020,000E)',
    -- 外键：关联检查
    StudyInstanceUID               VARCHAR(64) NOT NULL COMMENT '检查实例UID (外键) (0020,000D)',
    CONSTRAINT fk_series_study FOREIGN KEY (StudyInstanceUID) REFERENCES StudyEntity (StudyInstanceUID) ON DELETE CASCADE,
    -- 序列基本信息
    Modality                       VARCHAR(16) NOT NULL COMMENT '模态 (0008,0060) - CS, VM=1, max 16 chars (如 CT, MR, XR)',
    SeriesNumber                   INT COMMENT '序列号 (0020,0011) - IS, VM=1, max 12 chars → 用 INT',
    SeriesDate                     DATE COMMENT '序列日期 (0008,0021) - DA',
    SeriesTime                     TIME COMMENT '序列时间 (0008,0031) - TM',
    SeriesDescription              VARCHAR(64) COMMENT '序列描述 (0008,103E) - LO, VM=1, max 64 chars',
    -- 成像参数
    BodyPartExamined               VARCHAR(16) COMMENT '检查部位 (0018,0015) - CS, VM=1, max 16 chars',
    ProtocolName                   VARCHAR(64) COMMENT '协议名称 (0018,1030) - LO, VM=1, max 64 chars',
    ImageType                      VARCHAR(64) COMMENT '图像类型 (0008,0008) - CS, VM=1-n, max 16×n → 取64',
    AcquisitionNumber              INT COMMENT '采集号 (0020,0012) - IS, VM=1, max 12 chars → INT',
    AcquisitionTime                TIME COMMENT '采集时间 (0008,0032) - TM',
    AcquisitionDate                DATE COMMENT '采集日期 (0008,0022) - DA',
    -- 其他信息
    PerformingPhysicianName        VARCHAR(192) COMMENT '执行医生姓名 (0008,1050) - PN, 可继承自 Study',
    OperatorsName                  VARCHAR(192) COMMENT '操作员姓名 (0008,1070) - PN, VM=1-n',
    SeriesComments                 TEXT COMMENT '序列注释 (0040,4000) - LT, VM=1, max 10240 chars',
    -- 图像统计（可选）
    NumberOfSeriesRelatedInstances INT COMMENT '该序列关联的图像数量 (0020,1209) - IS',
    -- 时间戳
    CreatedTime                    DATETIME DEFAULT CURRENT_TIMESTAMP,
    UpdatedTime                    DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    -- 索引
    INDEX                          idx_study_uid (StudyInstanceUID),
    INDEX                          idx_modality (Modality),
    INDEX                          idx_series_number (SeriesNumber),
    INDEX                          idx_body_part (BodyPartExamined)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='序列实体表';

CREATE TABLE ImageEntity
(
    tenant_id                              VARCHAR(64) NOT NULL COMMENT '租户ID',
    -- 主键：图像唯一实例标识符 (SOP Instance UID)
    SOPInstanceUID                         VARCHAR(64) NOT NULL PRIMARY KEY COMMENT 'SOP实例UID (0008,0018)',

    -- 外键：关联上层实体
    SeriesInstanceUID                      VARCHAR(64) NOT NULL COMMENT '序列实例UID (外键，关联 SeriesEntity) (0020,000E)',
    StudyInstanceUID                       VARCHAR(64) NOT NULL COMMENT '检查实例UID (冗余但常用，便于查询) (0020,000D)',
    PatientID                              VARCHAR(64) NOT NULL COMMENT '患者ID (冗余但常用) (0010,0020)',

    -- 建立外键约束
    CONSTRAINT fk_image_series
        FOREIGN KEY (SeriesInstanceUID) REFERENCES SeriesEntity (SeriesInstanceUID) ON DELETE CASCADE,
    CONSTRAINT fk_image_study
        FOREIGN KEY (StudyInstanceUID) REFERENCES StudyEntity (StudyInstanceUID) ON DELETE CASCADE,
    CONSTRAINT fk_image_patient
        FOREIGN KEY (PatientID) REFERENCES PatientEntity (PatientID) ON DELETE CASCADE,
    -- 图像基本信息
    InstanceNumber                         INT COMMENT '实例编号 (0020,0013) - IS, VM=1, max 12 chars → INT',
    ImageComments                          TEXT COMMENT '图像注释 (0020,4000) - LT, VM=1, max 10240 chars',
    ContentDate                            DATE COMMENT '内容日期 (0008,0023) - DA',
    ContentTime                            TIME COMMENT '内容时间 (0008,0033) - TM',
    AcquisitionDateTime                    DATETIME COMMENT '采集日期时间 (0008,002A) - DT, 合并 DA+TM',

    -- 图像类型与分类
    ImageType                              VARCHAR(64) COMMENT '图像类型 (0008,0008) - CS, VM=1-n, e.g., ORIGINAL/PRIMARY/M/A',
    ImageOrientationPatient                VARCHAR(128) COMMENT '图像方向(患者) (0020,0037) - DS, VM=6, e.g., "1\0\0\0\1\0"',
    ImagePositionPatient                   VARCHAR(128) COMMENT '图像位置(患者) (0020,0032) - DS, VM=3, e.g., "-128.0\-128.0\98.0"',
    SliceThickness                         DECIMAL(8, 4) COMMENT '层厚 (mm) (0018,0050) - DS, VM=1',
    SpacingBetweenSlices                   DECIMAL(8, 4) COMMENT '层间距 (mm) (0018,0088) - DS, VM=1',
    SliceLocation                          DECIMAL(8, 4) COMMENT '切片位置 (0020,1041) - DS, VM=1',

    -- 像素与图像参数
    SamplesPerPixel                        TINYINT COMMENT '每像素样本数 (0028,0002) - US, VM=1, e.g., 1=灰度, 3=彩色',
    PhotometricInterpretation              VARCHAR(16) COMMENT '光度学解释 (0028,0004) - CS, VM=1, e.g., MONOCHROME2, RGB',
    Width                                  SMALLINT COMMENT '行数 (0028,0010) - US, VM=1, max 65535',
    Columns                                SMALLINT COMMENT '列数 (0028,0011) - US, VM=1, max 65535',
    BitsAllocated                          TINYINT COMMENT '分配位数 (0028,0100) - US, VM=1, e.g., 8, 16',
    BitsStored                             TINYINT COMMENT '存储位数 (0028,0101) - US, VM=1',
    HighBit                                TINYINT COMMENT '最高有效位 (0028,0102) - US, VM=1',
    PixelRepresentation                    TINYINT COMMENT '像素表示 (0028,0103) - US, VM=1, 0=无符号, 1=有符号',
    RescaleIntercept                       DECIMAL(8, 4) COMMENT '重缩放截距 (0028,1052) - DS, VM=1',
    RescaleSlope                           DECIMAL(8, 4) COMMENT '重缩放斜率 (0028,1053) - DS, VM=1',
    RescaleType                            VARCHAR(16) COMMENT '重缩放类型 (0028,1054) - LO, VM=1, e.g., "HU" for CT',

    -- 图像来源与设备
    AcquisitionDeviceProcessingDescription VARCHAR(64) COMMENT '采集设备处理描述 (0018,1010) - LO, VM=1',
    AcquisitionDeviceProcessingCode        VARCHAR(16) COMMENT '采集设备处理代码 (0018,1012) - SH, VM=1',
    DeviceSerialNumber                     VARCHAR(16) COMMENT '设备序列号 (0018,1000) - LO, VM=1, max 64 → 取16',
    SoftwareVersions                       VARCHAR(64) COMMENT '软件版本 (0018,1020) - LO, VM=1-n → 取64',

    -- 存储与引用信息
    TransferSyntaxUID                      VARCHAR(64) COMMENT '传输语法UID (0002,0010) - UI, e.g., 压缩或未压缩格式',
    PixelDataLocation                      VARCHAR(255) COMMENT '像素数据存储路径/URL (非DICOM Tag，业务扩展)',
    ThumbnailLocation                      VARCHAR(255) COMMENT '缩略图路径 (业务扩展)',

    -- 状态与元信息
    SOPClassUID                            VARCHAR(64) NOT NULL COMMENT 'SOP类UID (0008,0016) - UI, e.g., CT Image Storage',
    ImageStatus                            VARCHAR(16) DEFAULT 'ACTIVE' COMMENT '图像状态 (业务扩展: ACTIVE/ARCHIVED/DELETED)',

    -- 时间戳
    CreatedTime                            DATETIME    DEFAULT CURRENT_TIMESTAMP,
    UpdatedTime                            DATETIME    DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    -- 索引：优化查询性能
    INDEX                                  idx_series_uid (SeriesInstanceUID),
    INDEX                                  idx_study_uid (StudyInstanceUID),
    INDEX                                  idx_patient_id (PatientID),
    INDEX                                  idx_sop_class_uid (SOPClassUID),
    INDEX                                  idx_instance_number (InstanceNumber),
    INDEX                                  idx_content_date (ContentDate)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='图像实体表，存储单帧DICOM图像实例';