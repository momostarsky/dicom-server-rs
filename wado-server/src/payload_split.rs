use actix_web::HttpResponse;
use slog::error;

async fn split(mut payload: actix_web::web::Payload, boundary: &str) {
    use actix_web::web;
    use futures_util::stream::StreamExt;
    let mut buffer = Vec::with_capacity(512*1024*1024 );
    // 构造边界字符串
    let boundary_marker = format!("--{}", boundary);
    let end_boundary_marker = format!("--{}--", boundary);

    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(data) => {
                // TODO: data 是否包含完整的边界？
                buffer.extend_from_slice(&data);
            }
            Err(e) => {


            }
        }
    }

}

/// 检查数据是否包含完整边界
fn contains_complete_boundary(data: &[u8], boundary: &str) -> (Option<usize>, Option<usize>) {
    let start_boundary = format!("--{}", boundary);
    let end_boundary = format!("--{}--", boundary);

    // 查找起始边界位置
    let start_pos = data.windows(start_boundary.len())
        .position(|window| window == start_boundary.as_bytes());

    // 查找结束边界位置
    let end_pos = data.windows(end_boundary.len())
        .position(|window| window == end_boundary.as_bytes());
    (start_pos, end_pos)
}
