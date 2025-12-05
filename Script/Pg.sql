
drop table if exists dicom_state_meta;
create table dicom_state_meta
(
    tenant_id                varchar(64) not null,
    patient_id               varchar(64) not null,
    study_uid                varchar(64) not null,
    series_uid               varchar(64) not null,
    study_uid_hash           varchar(20) not null,
    series_uid_hash          varchar(20) not null,
    patient_name             varchar(64),
    patient_sex              varchar(1),
    patient_birth_date       date,
    patient_birth_time       time,
    patient_age              varchar(16),
    patient_size             double precision,
    patient_weight           double precision,
    pregnancy_status         integer,
    study_date               date        not null,
    study_date_origin        varchar(8)  not null,
    study_time               time,
    accession_number         varchar(16),
    study_id                 varchar(16),
    study_description        varchar(64),
    modality                 varchar(16),
    series_number            integer,
    series_date              date,
    series_time              time,
    series_description       varchar(256),
    body_part_examined       varchar(64),
    protocol_name            varchar(64),
    series_related_instances integer,
    created_time             timestamp,
    updated_time             timestamp,
    primary key (tenant_id, study_uid, series_uid)
);


create unique index index_state_unique
    on dicom_state_meta (tenant_id, study_uid, series_uid, accession_number);

create unique index index_state_unique4study
    on dicom_state_meta (tenant_id, study_uid,series_uid);

create unique index index_state_unique4patient
    on dicom_state_meta (tenant_id,patient_id, study_uid,series_uid);

drop table if exists dicom_json_meta;
create table  dicom_json_meta
(
    tenant_id                varchar(64) not null,
    study_uid                varchar(64) not null,
    series_uid               varchar(64) not null,
    study_uid_hash           varchar(20) not null,
    series_uid_hash          varchar(20) not null,
    study_date_origin        varchar(8)  not null,
    flag_time                timestamp   not null,
    created_time             timestamp   not null default current_timestamp(6),
    json_status              int         not null default 0,
    retry_times              int         not null default 0
);

ALTER TABLE dicom_json_meta
ADD CONSTRAINT PK_dicom_json_meta PRIMARY KEY (tenant_id, study_uid, series_uid);


--------------------------
---------------dicom_image_meta------------------
----------------------------
drop table if exists dicom_image_meta;
create table  dicom_image_meta
(
    tenant_id                  varchar(64) not null,
    patient_id                 varchar(64) not null,
    study_uid                  varchar(64) not null,
    series_uid                 varchar(64) not null,
    sop_uid                    varchar(64) not null,
    study_uid_hash             varchar(20) not null,
    series_uid_hash            varchar(20) not null,
    content_date               date,
    content_time               time,
    instance_number            integer,
    image_type                 varchar(128),
    image_orientation_patient  varchar(128),
    image_position_patient     varchar(64),
    slice_thickness            double precision,
    spacing_between_slices     double precision,
    slice_location             double precision,
    samples_per_pixel          integer,
    photometric_interpretation varchar(32),
    width                      integer,
    columns                    integer,
    bits_allocated             integer,
    bits_stored                integer,
    high_bit                   integer,
    pixel_representation       integer,
    rescale_intercept          double precision,
    rescale_slope              double precision,
    rescale_type               varchar(64),
    window_center              varchar(64),
    window_width               varchar(64),
    transfer_syntax_uid        varchar(64) not null,
    pixel_data_location        varchar(512),
    thumbnail_location         varchar(512),
    sop_class_uid              varchar(64) not null,
    image_status               varchar(32),
    space_size                 bigint,
    created_time               timestamp,
    updated_time               timestamp
);

comment on column dicom_image_meta.tenant_id is '租户ID';

comment on column dicom_image_meta.patient_id is '患者ID';

comment on column dicom_image_meta.study_uid is '检查UID';

comment on column dicom_image_meta.series_uid is '序列UID';

comment on column dicom_image_meta.sop_uid is '实例UID';

comment on column dicom_image_meta.study_uid_hash is '检查UID哈希值';

comment on column dicom_image_meta.series_uid_hash is '序列UID哈希值';

comment on column dicom_image_meta.content_date is '内容日期';

comment on column dicom_image_meta.content_time is '内容时间';

comment on column dicom_image_meta.instance_number is '实例编号';

comment on column dicom_image_meta.image_type is '图像类型';

comment on column dicom_image_meta.image_orientation_patient is '图像方向(患者坐标系)';

comment on column dicom_image_meta.image_position_patient is '图像位置(患者坐标系)';

comment on column dicom_image_meta.slice_thickness is '层厚';

comment on column dicom_image_meta.spacing_between_slices is '层间距';

comment on column dicom_image_meta.slice_location is '切片位置';

comment on column dicom_image_meta.samples_per_pixel is '每个像素采样数';

comment on column dicom_image_meta.photometric_interpretation is '光度解释';

comment on column dicom_image_meta.width is '图像行数';

comment on column dicom_image_meta.columns is '图像列数';

comment on column dicom_image_meta.bits_allocated is '分配位数';

comment on column dicom_image_meta.bits_stored is '存储位数';

comment on column dicom_image_meta.high_bit is '高比特位';

comment on column dicom_image_meta.pixel_representation is '像素表示法';

comment on column dicom_image_meta.rescale_intercept is '重建截距';

comment on column dicom_image_meta.rescale_slope is '重建斜率';

comment on column dicom_image_meta.rescale_type is '重建类型';

comment on column dicom_image_meta.window_center is '窗位中心';

comment on column dicom_image_meta.window_width is '窗宽';

comment on column dicom_image_meta.transfer_syntax_uid is '传输语法UID';

comment on column dicom_image_meta.pixel_data_location is '像素数据位置';

comment on column dicom_image_meta.thumbnail_location is '缩略图位置';

comment on column dicom_image_meta.sop_class_uid is 'SOP类UID';

comment on column dicom_image_meta.image_status is '图像状态';

comment on column dicom_image_meta.space_size is '占用空间大小';

comment on column dicom_image_meta.created_time is '创建时间';

comment on column dicom_image_meta.updated_time is '更新时间';


-- 方案2: 使用现有字段创建组合主键（需要先删除现有约束）
ALTER TABLE dicom_image_meta
    ADD CONSTRAINT pk_dicom_image_meta PRIMARY KEY (tenant_id, study_uid, series_uid, sop_uid);


---------------------------------
drop table if exists dicom_object_meta;
create table  dicom_object_meta
(
    trace_id            varchar(36)   not null,
    worker_node_id      varchar(64)   not null,
    tenant_id           varchar(64)   not null,
    patient_id          varchar(64)   not null,
    study_uid           varchar(64)   null,
    series_uid          varchar(64)   null,
    sop_uid             varchar(64)   null,
    file_size           bigint        null,
    file_path           varchar(512) null,
    transfer_syntax_uid varchar(64)   null,
    number_of_frames    int           null,
    series_uid_hash     VARCHAR(20)   null,
    study_uid_hash      VARCHAR(20)   null,
    accession_number    varchar(64)   null,
    target_ts           varchar(64)   null,
    study_date          date          null,
    transfer_status     varchar(64)   null,
    source_ip           varchar(24)   null,
    source_ae           varchar(64)   null,
    created_time        timestamp     not null default CURRENT_TIMESTAMP
);

comment on column dicom_object_meta.tenant_id      is '租户ID';
comment on column dicom_object_meta.patient_id     is '患者ID';
comment on column dicom_object_meta.trace_id       is '全局唯一追踪ID，作为主键';
comment on column dicom_object_meta.worker_node_id is '工作节点 ID';

ALTER TABLE dicom_object_meta   ADD CONSTRAINT pk_dicom_object_meta PRIMARY KEY  (trace_id);

create index idx_dicom_object_meta on dicom_object_meta (tenant_id, patient_id,study_uid,series_uid,sop_uid);
create index idx_dicom_object_meta_date on dicom_object_meta (tenant_id, study_date);
create index idx_dicom_object_meta_createdate on dicom_object_meta (tenant_id, created_time);

-----------------------收图记录-------------------------