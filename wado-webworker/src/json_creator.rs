use crate::AppState;
use common::dicom_json_helper::generate_series_json;
use common::server_config::WebWorkerConfig;
use database::dicom_dbprovider::current_time;
use database::dicom_dbtype::BoundedString;
use database::dicom_meta::DicomJsonMeta;
use slog::{error, info};
use std::ops::Sub;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::time::{Duration, interval};

// 后台任务管理器
pub(crate) async fn background_task_manager(app_state: AppState) {
    let mut interval = interval(Duration::from_secs(30)); // 每30秒检查一次
    let mut sys = System::new_all();

    let webworker = match &app_state.config.webworker {
        None => WebWorkerConfig {
            interval_minute: 5,
            cpu_usage: 50,
            memory_usage: 50,
        },
        Some(webworker) => webworker.clone(),
    };
    loop {
        interval.tick().await;

        // 刷新系统信息
        sys.refresh_all();

        // 获取CPU和内存使用率
        let cpu_usage = get_cpu_usage(&sys);
        let memory_usage = get_memory_usage(&sys);

        // 检查是否满足执行条件（CPU < 60% 且 内存 < 70%）
        if cpu_usage < webworker.cpu_usage as f32 && memory_usage < webworker.memory_usage as f32 {
            // 执行后台任务
            if let Err(e) =
                execute_background_json_generation(&app_state, webworker.interval_minute as i64)
                    .await
            {
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
    interval_minute: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        app_state.log,
        "Starting background JSON metadata generation"
    );

    let cd = current_time();
    let end_time = cd.sub(chrono::Duration::minutes(interval_minute));
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
        let tenant_id = record.tenant_id.as_str();
        let study_uid = record.study_uid.as_str();
        let series_uid = record.series_uid.as_str();

        match app_state
            .redis_helper
            .set_series_metadata_gererate(&tenant_id, &series_uid)
            .await
        {
            Ok(_) => {
                info!(
                    app_state.log,
                    "Set series metadata generate for study: {}, series: {}",
                    record.study_uid,
                    record.series_uid
                );
            }
            Err(e) => {
                error!(
                    app_state.log,
                    "Failed to set series metadata generate for study: {}, series: {}: {}",
                    record.study_uid,
                    record.series_uid,
                    e
                )
            }
        };
        // 这里应该调用实际的JSON生成逻辑
        // 可以参考wado_rs_controller.rs中的实现
        let result_status = match generate_series_json(&record).await {
            Ok(_) => {
                info!(
                    app_state.log,
                    "Generated JSON for study: {}, series: {}", record.study_uid, record.series_uid
                );
                1
            }
            Err(e) => {
                error!(
                    app_state.log,
                    "Failed to generate JSON for study: {}, series: {}: {}",
                    record.study_uid,
                    record.series_uid,
                    e
                );
                2
            }
        };
        json_mets.push(DicomJsonMeta {
            tenant_id: BoundedString::<64>::make_str(tenant_id),
            study_uid: BoundedString::<64>::make_str(study_uid),
            series_uid: BoundedString::<64>::make_str(series_uid),
            study_uid_hash: record.study_uid_hash,
            series_uid_hash: record.series_uid_hash,
            study_date_origin: record.study_date_origin,
            flag_time: record.updated_time,
            created_time: current_time(),
            json_status: result_status,
            retry_times: 1,
        });

        // 删除redis中的记录,无论生成成功与否
        match app_state
            .redis_helper
            .del_series_metadata_gererate(&tenant_id, &series_uid)
            .await
        {
            Ok(_) => {
                info!(
                    app_state.log,
                    "Set series metadata generate for study: {}, series: {}", study_uid, series_uid
                );
            }
            Err(e) => {
                error!(
                    app_state.log,
                    "Failed to set series metadata generate for study: {}, series: {}: {}",
                    study_uid,
                    series_uid,
                    e
                )
            }
        }
    }

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

    info!(
        app_state.log,
        "Background JSON metadata generation completed"
    );
    Ok(())
}
