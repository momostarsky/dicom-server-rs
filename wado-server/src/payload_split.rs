use actix_web::HttpResponse;
use actix_web::error::PayloadError;
use axum::Error;
use bytes::Bytes;
use futures_util::StreamExt;
use slog::error;
#[derive(Debug, Clone)]
pub struct PayloadSplit {
    content_type: String,
    payload: bytes::Bytes,
}
async fn split(
    mut payload: actix_web::web::Payload,
    boundary: &str,
) -> Result< Vec<PayloadSplit>, PayloadError> {
    let mut buffer = Vec::with_capacity(512 * 1024 * 1024);
    let json_boundary_marker = format!("--{}\r\nContent-Type: application/json\r\n\r\n", boundary);
    let dicm_boundary_marker = format!("--{}\r\nContent-Type: application/dicom\r\n\r\n", boundary);

    let end_boundary = format!("\r\n--{}\r\n", boundary);
    let over_boundary = format!("\r\n--{}--\r\n", boundary);
    let start_boundary = format!("\r\n--{}\r\n", boundary);

    let mut entities: Vec<PayloadSplit> = Vec::new();
    loop {
        match payload.next().await {
            Some(chunk) => match chunk {
                Ok(bytes) => {
                    buffer.extend_from_slice(&bytes);
                    // 开始检查是否包含完整的边界
                    //TODO: 如果buffer 以over_boundary 结尾，说明已经结束，可以直接退出循环
                    if buffer.ends_with(over_boundary.as_bytes()) {
                        //TODO:直接用 \r\n--%s\r\n  进行分割

                        break;
                    }

                }
                Err(e) => {
                    error!(
                        slog::Logger::root(slog::Discard, slog::o!()),
                        "Error reading payload chunk: {}", e
                    );
                    return Err(e);
                }
            },
            None => {
                error!(
                    slog::Logger::root(slog::Discard, slog::o!()),
                    "Payload stream ended unexpectedly"
                );
                return Err(PayloadError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Payload stream ended unexpectedly",
                )));

            }
        }
    }
    Ok(entities)
}

/// 检查数据是否包含完整边界
fn contains_complete_boundary(data: &[u8], boundary: &str) -> (Option<usize>, Option<usize>) {
    let start_boundary = format!("--{}", boundary);
    let end_boundary = format!("--{}--", boundary);

    // 查找起始边界位置
    let start_pos = data
        .windows(start_boundary.len())
        .position(|window| window == start_boundary.as_bytes());

    // 查找结束边界位置
    let end_pos = data
        .windows(end_boundary.len())
        .position(|window| window == end_boundary.as_bytes());
    (start_pos, end_pos)
}
/// 生成模拟的 multipart/related 数据流用于测试
pub fn generate_test_multipart_data(boundary: &str) -> Vec<u8> {
    let mut data = Vec::new();

    // 1. JSON 部分的边界和头部
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
    data.extend_from_slice(b"{\"patient_id\": \"P123\", \"study_uid\": \"1.2.3.4.5\"}");

    // 2. 第一个 DICOM 文件部分
    data.extend_from_slice(b"\r\n");
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Type: application/dicom\r\n\r\n");
    // 模拟 DICOM 数据 (包含 DICM 标记)
    data.extend_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
    0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
    0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
    0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0D.I.C.M......"); // 简化的 DICOM 数据

    // 3. 第二个 DICOM 文件部分
    data.extend_from_slice(b"\r\n");
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Type: application/dicom\r\n\r\n");
    data.extend_from_slice(b"......D.I.C.M......"); // 简化的 DICOM 数据

    // 4. 结束边界
    data.extend_from_slice(b"\r\n");
    data.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    data
}

/// 生成更真实的测试数据，包含有效的 DICOM 头部
pub fn generate_realistic_test_data(boundary: &str) -> Vec<u8> {
    let mut data = Vec::new();

    // JSON 部分
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
    data.extend_from_slice(br#"{"patient_id": "TEST001", "study_uid": "1.3.6.1.4.1.55555.3.1"}"#);

    // DICOM 部分 - 包含有效的 DICM 标记
    data.extend_from_slice(b"\r\n");
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Type: application/dicom\r\n\r\n");

    // 构建最小的 DICOM 文件头
    let mut dicom_data = vec![0u8; 128]; // 128 字节 preamble
    dicom_data.extend_from_slice(b"DICM"); // DICM 标记
    // 添加一些基本的 DICOM 元素...
    data.extend_from_slice(&dicom_data);

    // 结束边界
    data.extend_from_slice(b"\r\n");
    data.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_multipart_data() {
        let boundary = "TEST_BOUNDARY";
        let data = generate_test_multipart_data(boundary);

        // 验证数据包含起始边界
        let start_boundary = format!("--{}", boundary);
        assert!(String::from_utf8_lossy(&data).contains(&start_boundary));

        // 验证数据包含结束边界
        let end_boundary = format!("--{}--", boundary);
        assert!(String::from_utf8_lossy(&data).contains(&end_boundary));

        // 验证包含 Content-Type 头部
        assert!(String::from_utf8_lossy(&data).contains("Content-Type: application/json"));
        assert!(String::from_utf8_lossy(&data).contains("Content-Type: application/dicom"));
    }

    #[test]
    fn test_contains_complete_boundary() {
        let boundary = "DICOM_BOUNDARY";
        let data = generate_test_multipart_data(boundary);
        let (start_pos, end_pos) = contains_complete_boundary(&data, boundary);

        // 验证找到了边界位置
        assert!(start_pos.is_some());
        assert!(end_pos.is_some());
    }
}
