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
    accession_number         varchar(16) not null,
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
create primary key  PK_dicom_json_meta(tenant_id, study_uid, series_uid)
    on  dicom_json_meta (tenant_id, study_uid, series_uid);


