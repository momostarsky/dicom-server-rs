use crate::AppState;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region, SharedCredentialsProvider};
use aws_sdk_s3::error::SdkError;
use aws_smithy_types::byte_stream::ByteStream;
use common::dicom_json_helper::generate_series_json;
use common::storage_config::StorageConfig;
use common::utils::{collect_dicom_file, get_current_time};
use database::dicom_dbtype::BoundedString;
use database::dicom_meta::DicomStateArchive;
use futures_util::future::join_all;
use slog::{error, info};
use std::fs;
use std::time::Duration;
use tokio::time::interval;

pub(crate) async fn background_task_manager(app_state: AppState) {
    info!(app_state.log, "Starting background task manager");

    if app_state.config.s3config.is_none() {
        info!(
            app_state.log,
            "S3 configuration not found, skipping background archive task"
        );
        return;
    }
    let mut interval = interval(Duration::from_mins(10)); // 每10分钟检查一次
    loop {
        interval.tick().await;

        if let Err(e) = execute_archive(&app_state).await {
            error!(app_state.log, "Background JSON generation failed: {}", e);
        }
    }
}

/// execute archive dicom files to s3 storage
async fn execute_archive(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!(app_state.log, "Starting background execute_archive");

    // 调用数据库获取需要生成JSON的记录
    let pending_records = app_state
        .db
        .get_state_archives()
        .await
        .map_err(|e| format!("Failed to get dicomState to archiving records: {}", e))?;
    info!(
        app_state.log,
        "Found {} records for archive processing",
        pending_records.len()
    );

    let access_key = app_state
        .config
        .s3config
        .as_ref()
        .unwrap()
        .s3_access_key
        .clone();
    let secret_key = app_state
        .config
        .s3config
        .as_ref()
        .unwrap()
        .s3_secret_key
        .clone();
    let endpoint_url = app_state
        .config
        .s3config
        .as_ref()
        .unwrap()
        .s3_end_point
        .clone();
    // 修复 region 取值可能为空的问题
    let region = app_state
        .config
        .s3config
        .as_ref()
        .unwrap()
        .region
        .clone()
        .unwrap_or_else(|| "us-east-1".to_string()); // 提供默认region值
    let dcm_bucket_name = app_state
        .config
        .s3config
        .as_ref()
        .unwrap()
        .s3_dicom_bucket_name
        .clone();
    let json_bucket_name = app_state
        .config
        .s3config
        .as_ref()
        .unwrap()
        .s3_json_bucket_name
        .clone();

    // 2. 设置凭证和配置
    let credentials = Credentials::new(access_key, secret_key, None, None, "manual");
    let config = aws_config::from_env()
        .region(Region::new(region))
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .endpoint_url(endpoint_url)
        .load()
        .await;
    // 注意：必须开启 force_path_style 以兼容 MinIO
    let s3_config_builder = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();
    let client = Client::from_conf(s3_config_builder);

    let local_storage = StorageConfig::make_storage_config(&app_state.config);
    let mut success_uploads = vec![];
    for record in pending_records {
        info!(app_state.log, "Processing archive record: {:?}", record);
        let (tenant_id, study_uid, series_uid) = record;

        let json_file_path =
            local_storage.json_metadata_path_for_series(&tenant_id, &study_uid, &series_uid, false);
        if json_file_path.is_err() {
            error!(
                app_state.log,
                "Failed to get json file path: {}",
                json_file_path.err().unwrap()
            );
            continue;
        }
        let json_file_path = json_file_path?.clone();
        if !fs::exists(&json_file_path).is_ok() {
            error!(
                app_state.log,
                "json file path: {} not exists", &json_file_path
            );
            let result_status =
                match generate_series_json(&tenant_id, &study_uid, &series_uid).await {
                    Ok(_) => {
                        info!(
                            app_state.log,
                            "Generated JSON for tenant_id:{},  study: {}, series: {}",
                            &tenant_id,
                            &study_uid,
                            &series_uid
                        );
                        true
                    }
                    Err(e) => {
                        error!(
                            app_state.log,
                            "Failed to generate JSON for tenant_id:{},   study: {}, series: {}: {}",
                            &tenant_id,
                            &study_uid,
                            &series_uid,
                            e
                        );
                        false
                    }
                };
            if false == result_status {
                continue;
            }
        }
        let series_dir =
            local_storage.make_series_dicom_dir(&tenant_id, &study_uid, &series_uid, false);
        if series_dir.is_err() {
            error!(
                app_state.log,
                "Failed to get series directory: {}",
                series_dir.err().unwrap()
            );
            continue;
        }
        let series_dir = series_dir?.clone();
        match fs::exists(&series_dir) {
            Ok(true) => {}
            _ => {
                error!(
                    app_state.log,
                    "Series directory does not exist: {}", &series_dir
                );
                continue;
            }
        }
        // upload json file
        let json_object_key = format!("{}/{}/{}.json", tenant_id, study_uid, series_uid);
        info!(app_state.log, "Uploading to MinIO: {}", &json_object_key);
        let file_contents = match fs::read(&json_file_path) {
            Ok(contents) => contents,
            Err(e) => {
                error!(
                    app_state.log,
                    "Failed to read DICOM file {}: {}", &json_file_path, e
                );
                continue;
            }
        };
        match client
            .put_object()
            .bucket(&json_bucket_name)
            .key(&json_object_key)
            .body(ByteStream::from(file_contents))
            .content_type("application/json") // 设置正确的 MIME 类型
            .send()
            .await
        {
            Ok(_) => {
                info!(
                    app_state.log,
                    "Successfully uploaded {} to MinIO!", &json_object_key
                );
            }
            Err(e) => {
                error!(
                    app_state.log,
                    "Failed to upload {} to MinIO: {}", &json_object_key, e
                );
            }
        }
        let mut dicom_files = vec![];
        collect_dicom_file((&series_dir).as_ref(), &mut dicom_files);
        let mut upload_tasks = Vec::new();
        let start_time = get_current_time();
        let mut space_size = 0i64;
        for dicom_file in dicom_files {
            info!(app_state.log, "Processing dicom file: {:?}", dicom_file);

            // 构建对象键，去除绝对路径部分，只保留相对路径
            let relative_path = dicom_file.strip_prefix(&series_dir).unwrap_or(&dicom_file);

            let object_key = format!(
                "{}/{}/{}/{}",
                tenant_id,
                study_uid,
                series_uid,
                relative_path
                    .to_string_lossy()
                    .strip_prefix('/')
                    .unwrap_or(&relative_path.to_string_lossy())
            );
            info!(app_state.log, "Uploading to MinIO: {}", &object_key);
            let file_contents = match fs::read(&dicom_file) {
                Ok(contents) => contents,
                Err(e) => {
                    error!(
                        app_state.log,
                        "Failed to read DICOM file {}: {}",
                        &dicom_file.display(),
                        e
                    );
                    continue;
                }
            };

            space_size = space_size + file_contents.len() as i64;
            let client_clone = client.clone(); // 需要Clone client
            let bucket_name = dcm_bucket_name.clone();
            let key = object_key.clone();

            let upload_task = async move {
                match client_clone
                    .put_object()
                    .bucket(&bucket_name)
                    .key(&key)
                    .body(ByteStream::from(file_contents))
                    .content_type("application/dicom")
                    .send()
                    .await
                {
                    Ok(_) => {
                        println!("Successfully uploaded {}", key);
                        Ok::<(), SdkError<_>>(())
                    }
                    Err(e) => {
                        eprintln!("Failed to upload {}: {}", key, e);
                        Err(e)
                    }
                }
            };

            upload_tasks.push(upload_task);
        }
        let results = join_all(upload_tasks).await;
        for result in results {
            if let Err(e) = result {
                error!(app_state.log, "Upload error: {}", e);
            }
        }
        let end_time = get_current_time();
        success_uploads.push(DicomStateArchive {
            tenant_id: BoundedString::<64>::make_str(&tenant_id),
            study_uid: BoundedString::<64>::make_str(&study_uid),
            series_uid: BoundedString::<64>::make_str(&series_dir),
            start_time,
            end_time,
            space_size,
        });
    }
    match app_state.db.save_archive_list(&success_uploads).await {
        Ok(_) => {
            info!(app_state.log, "Archive list saved to database");
        }
        Err(e) => {
            error!(
                app_state.log,
                "Failed to save archive list to database: {}", e
            );
        }
    }
    info!(app_state.log, "Background archive completed");
    Ok(())
}
