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
    "max_error_number" = "1000"
)
FROM KAFKA (
    "kafka_broker_list" = "192.168.1.14:9092",
    "kafka_topic" = "log_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);


CREATE ROUTINE LOAD medical_state_load ON dicom_state_meta
COLUMNS (
        tenant_id  ,
        patient_id ,
        study_uid ,
        series_uid,
        study_uid_hash,
        series_uid_hash,
        study_date_origin,

        -- 患者信息
        patient_name,
        patient_sex ,
        patient_birth_date ,
        patient_birth_time,
        patient_age,
        patient_size,
        patient_weight,
        medical_alerts,
        allergies,
        pregnancy_status,
        occupation,

        -- 检查信息
        study_date,
        study_time,
        accession_number,
        study_id,
        study_description,
        referring_name,
        admission_id,
        performing_name,

        -- 序列信息
        modality,
        series_number,
        series_date,
        series_time,
        series_description,
        body_part_examined,
        protocol_name,
        operators_name,
        manufacturer,
        institution_name,
        device_serial_number,
        software_versions,
        series_related_instances,
        created_time,
        updated_time = NOW()
)
PROPERTIES (
    "desired_concurrent_number" = "3",
    "max_batch_interval" = "10",
    "max_batch_rows" = "300000",
    "max_batch_size" = "209715200",
    "format" = "json",
    "max_error_number" = "1000",
    "strip_outer_array" = "false"
)
FROM KAFKA (
    "kafka_broker_list" = "192.168.1.14:9092",
    "kafka_topic" = "dicom_state_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);



CREATE ROUTINE LOAD medical_image_load ON dicom_image_meta
COLUMNS (
  tenant_id ,
  patient_id ,
  study_uid  ,
  series_uid  ,
  sop_uid ,
  study_uid_hash ,
  series_uid_hash  ,
  study_date_origin  ,
  instance_number ,
  content_date ,
  content_time  ,
  image_type ,
  image_orientation_patient ,
  image_position_patient ,
  slice_thickness ,
  spacing_between_slices  ,
  slice_location  ,
  samples_per_pixel ,
  photometric_interpretation  ,
  width  ,
  `columns`  ,
  bits_allocated  ,
  bits_stored  ,
  high_bit ,
  pixel_representation ,
  rescale_intercept ,
  rescale_slope ,
  rescale_type ,
  window_center  ,
  window_width  ,
  transfer_syntax_uid ,
  pixel_data_location  ,
  thumbnail_location  ,
  sop_class_uid  ,
  image_status  ,
  space_size ,
  created_time  ,
  updated_time = NOW()
)
PROPERTIES (
    "desired_concurrent_number" = "3",

    "max_batch_interval" = "10",
    "max_batch_rows" = "300000",
    "max_batch_size" = "209715200",
    "format" = "json",
    "max_error_number" = "1000",
    "strip_outer_array" = "false"

)
FROM KAFKA (
    "kafka_broker_list" = "192.168.1.14:9092",
    "kafka_topic" = "dicom_image_queue",
    "kafka_partitions" = "0",
    "property.kafka_default_offsets" = "OFFSET_BEGINNING"
);

