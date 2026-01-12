use crate::AppState;
use slog::{error, info};
use std::time::Duration; 
use tokio::time::interval;

pub(crate) async fn background_task_manager(app_state: AppState) {
    let mut interval = interval(Duration::from_mins(10)); // 每10分钟检查一次
    loop {
        interval.tick().await;

        if let Err(e) = execute_archive(&app_state).await {
            error!(app_state.log, "Background JSON generation failed: {}", e);
        }
    }
}

// 执行后台JSON生成任务
async fn execute_archive(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        app_state.log,
        "Starting background JSON metadata generation"
    );

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

    info!(app_state.log, "Background archive completed");
    Ok(())
}
