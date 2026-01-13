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

/// execute archive dicom files to s3 storage
async fn execute_archive(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        app_state.log,
        "Starting background execute_archive"
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
    for record in pending_records {
         info!(app_state.log, "Processing archive record: {:?}", record);
    }

    info!(app_state.log, "Background archive completed");
    Ok(())
}
