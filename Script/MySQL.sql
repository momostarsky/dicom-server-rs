create table dicomdb.dicom_state_meta
(
    tenant_id                varchar(64)  not null,
    patient_id               varchar(64)  not null,
    study_uid                varchar(64)  not null,
    series_uid               varchar(64)  not null,
    study_uid_hash           char(20)     not null,
    series_uid_hash          char(20)     not null,
    patient_name             varchar(64)  null,
    patient_sex              varchar(1)   null,
    patient_birth_date       date         null,
    patient_birth_time       time         null,
    patient_age              varchar(16)  null,
    patient_size             double       null,
    patient_weight           double       null,
    pregnancy_status         int          null,
    study_date               date         not null,
    study_date_origin        char(8)      not null,
    study_time               time         null,
    accession_number         varchar(16)  not null,
    study_id                 varchar(16)  null,
    study_description        varchar(64)  null,
    modality                 varchar(16)  null,
    series_number            int          null,
    series_date              date         null,
    series_time              time         null,
    series_description       varchar(256) null,
    body_part_examined       varchar(64)  null,
    protocol_name            varchar(64)  null,
    series_related_instances int          null,
    created_time             datetime(6)  null,
    updated_time             datetime(6)  null,
    primary key (tenant_id, study_uid, series_uid),
    constraint index_state_unique unique (tenant_id, study_uid, series_uid, accession_number)
);


drop table if exists dicom_json_meta;
create table  dicom_json_meta
(
    tenant_id                varchar(64) not null,
    study_uid                varchar(64) not null,
    series_uid               varchar(64) not null,
    study_uid_hash           varchar(20) not null,
    series_uid_hash          varchar(20) not null,
    study_date_origin        varchar(8)  not null,
    flag_time                datetime(6)   not null,
    created_time             datetime(6)   not null default current_timestamp(6),
    json_status              int         not null default 0,
    retry_times              int         not null default 0
);
create primary key  PK_dicom_json_meta(tenant_id, study_uid, series_uid)
    on  dicom_json_meta (tenant_id, study_uid, series_uid);