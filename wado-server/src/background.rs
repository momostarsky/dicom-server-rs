use crate::AppState;
use database::dicom_dbprovider::{DbError, current_time};
use database::dicom_meta::{DicomJsonMeta, DicomStateMeta};
use slog::{error, info};
use std::ops::Sub;
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};
use tokio::time::{Duration, interval};

// 后台任务管理器
pub(crate) async fn background_task_manager(app_state: AppState) {
    let mut interval = interval(Duration::from_secs(30)); // 每30秒检查一次
    let mut sys = System::new_all();

    loop {
        interval.tick().await;

        // 刷新系统信息
        sys.refresh_all();

        // 获取CPU和内存使用率
        let cpu_usage = get_cpu_usage(&sys);
        let memory_usage = get_memory_usage(&sys);

        // 检查是否满足执行条件（CPU < 60% 且 内存 < 70%）
        if cpu_usage < 60.0 && memory_usage < 70.0 {
            // 执行后台任务
            if let Err(e) = execute_background_json_generation(&app_state).await {
                error!(app_state.log, "Background JSON generation failed: {}", e);
            }
        } else {
            info!(
                app_state.log,
                "System busy - CPU: {:.2}%, Memory: {:.2}%", cpu_usage, memory_usage
            );
        }
    }
}

// 获取CPU使用率
fn get_cpu_usage(sys: &System) -> f32 {
    let cpus = sys.cpus();
    if cpus.is_empty() {
        return 0.0;
    }

    let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    total_usage / cpus.len() as f32
}

// 获取内存使用率
fn get_memory_usage(sys: &System) -> f32 {
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();

    if total_memory == 0 {
        0.0
    } else {
        (used_memory as f32 / total_memory as f32) * 100.0
    }
}

// 执行后台JSON生成任务
async fn execute_background_json_generation(
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        app_state.log,
        "Starting background JSON metadata generation"
    );

    let cd = current_time();
    let end_time = cd.sub(chrono::Duration::minutes(3));
    // 调用数据库获取需要生成JSON的记录
    let pending_records = app_state
        .db
        .get_json_metaes(end_time)
        .await
        .map_err(|e| format!("Failed to get pending JSON metadata records: {}", e))?;

    info!(
        app_state.log,
        "Found {} records for JSON generation",
        pending_records.len()
    );

    let mut json_mets = vec![];
    // 逐个处理记录
    for record in pending_records {
        // 这里应该调用实际的JSON生成逻辑
        // 可以参考wado_rs_controller.rs中的实现
        match generate_json_for_record(app_state, &record).await {
            Ok(_) => {
                info!(
                    app_state.log,
                    "Generated JSON for study: {}, series: {}", record.study_uid, record.series_uid
                );
                json_mets.push(DicomJsonMeta {
                    tenant_id: record.tenant_id,
                    study_uid: record.study_uid.clone(),
                    series_uid: record.series_uid.clone(),
                    study_uid_hash: record.study_uid_hash,
                    series_uid_hash: record.series_uid_hash,
                    study_date_origin: record.study_date_origin,
                    flag_time: record.updated_time,
                    created_time: current_time(),
                    json_status: 1,
                    retry_times: 1,
                });
            }
            Err(e) => {
                error!(
                    app_state.log,
                    "Failed to generate JSON for study: {}, series: {}: {}",
                    record.study_uid,
                    record.series_uid,
                    e
                );
                json_mets.push(DicomJsonMeta {
                    tenant_id: record.tenant_id,
                    study_uid: record.study_uid.clone(),
                    series_uid: record.series_uid.clone(),
                    study_uid_hash: record.study_uid_hash,
                    series_uid_hash: record.series_uid_hash,
                    study_date_origin: record.study_date_origin,
                    flag_time: record.updated_time,
                    created_time: current_time(),
                    json_status: 2,
                    retry_times: 1,
                });
            }
        }
    }
    if !json_mets.is_empty() {
        match app_state.db.save_json_list(&json_mets).await {
            Ok(_) => {
                info!(
                    app_state.log,
                    "Saved {} JSON metadata records",
                    json_mets.len()
                );
            }
            Err(_) => {
                error!(app_state.log, "Failed to save JSON metadata records");
            }
        }
    }

    info!(
        app_state.log,
        "Background JSON metadata generation completed"
    );
    Ok(())
}

// 为单个记录生成JSON
async fn generate_json_for_record(
    app_state: &AppState,
    record: &DicomStateMeta,
) -> Result<(), Box<dyn std::error::Error>> {
    // 实现具体的JSON生成逻辑
    // 这里应该参考retrieve_series_metadata等函数的实现
    info!(
        app_state.log,
        "Generating JSON for study: {}, series: {}", record.study_uid, record.series_uid
    );

    // TODO: 实现实际的JSON生成逻辑
    // 可以调用dicom_json_helper中的相关函数

    Ok(())
}
