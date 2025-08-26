use dicom_object;
use dicom_object::OpenFileOptions;
use gdcm_conv::PhotometricInterpretation;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub enum ChangeStatus {
    FileReadError(String),
    FileWriteError(String),
    ConversionError(String),
}

impl std::fmt::Display for ChangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeStatus::FileReadError(msg) => write!(f, "File read error: {}", msg),
            ChangeStatus::FileWriteError(msg) => write!(f, "File write error: {}", msg),
            ChangeStatus::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}
impl std::error::Error for ChangeStatus {}

pub async fn convert_ts_with_pixel_data(
    src_file: &str,
    file_size: usize,
    output_path: &str,
    overwrite: bool,
) -> Result<(), ChangeStatus> {
    // 步骤 1: 读取 DICOM 文件
    let obj = match OpenFileOptions::new().open_file(src_file) {
        Ok(obj) => obj,
        Err(e) => {
            return Err(ChangeStatus::FileReadError(format!(
                "Failed to open file {}: {}",
                src_file, e
            )));
        }
    };
    //-------------创建一个空的向量，用于存储文件内容--长度为文件大小
    let mut allocated_size = file_size;
    if file_size == 0 {
        allocated_size = 512 * 512;
    }
    let mut input_buffer = Vec::with_capacity(allocated_size);
    // 将 DICOM 对象写入缓冲区,如果出错,则内存分配失败,直接退出
    match obj.write_all(&mut input_buffer) {
        Ok(_) => {}
        Err(e) => {
            return Err(ChangeStatus::FileWriteError(format!(
                "Failed to write to buffer: {}",
                e
            )));
        }
    };
    match gdcm_conv::pipeline(
        // Input DICOM file buffer
        input_buffer,
        // Estimated Length
        None,
        // First Transfer Syntax conversion
        gdcm_conv::TransferSyntax::RLELossless,
        // Photometric conversion
        PhotometricInterpretation::None,
        // Second Transfer Syntax conversion
        gdcm_conv::TransferSyntax::None,
    ) {
        Ok(buffer) => {
            let mut output_file = match File::create(&output_path) {
                Ok(file) => file,
                Err(e) => {
                    return Err(ChangeStatus::FileWriteError(format!(
                        "Failed to create output file {}: {}",
                        output_path, e
                    )));
                }
            };
            match output_file.write_all(&buffer) {
                Ok(_) => {
                    if overwrite {
                        match fs::copy(output_path, src_file) {
                            Ok(_) => {}
                            Err(_) => {
                                return Err(ChangeStatus::FileWriteError(format!(
                                    "Failed to copy  file from  {} to  {}",
                                    output_path, src_file
                                )));
                            }
                        }
                    } else {
                        println!("Conversion successful, output saved to {}", output_path);
                    }
                }
                Err(e) => {
                    return Err(ChangeStatus::FileWriteError(format!(
                        "Failed to write to output file {}: {}",
                        output_path, e
                    )));
                }
            }
        }
        Err(e) => {
            return Err(ChangeStatus::ConversionError(format!(
                "Conversion failed: {}",
                e
            )));
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[tokio::test]
    async fn test_convert_ts_with_pixel_data_success() {
        // 创建一个临时的测试文件
        let test_file = "./2.dcm";
        let output_file = "./2-X.dcm";

        // 获取文件大小
        let metadata = fs::metadata(test_file).unwrap();
        let file_size = metadata.len() as usize;

        // 调用函数
        let result = convert_ts_with_pixel_data(test_file, file_size, output_file,false).await;

        assert!(result.is_ok());
        // 验证结果
        // 注意：由于我们没有真正的DICOM文件和gdcm_conv库，这里可能会返回ConversionError
        // 但在实际环境中，如果有正确的DICOM文件，应该会成功

        // 清理测试文件
        // if Path::new(test_file).exists() {
        //     fs::remove_file(test_file).unwrap();
        // }
        // if Path::new(output_file).exists() {
        //     fs::remove_file(output_file).unwrap();
        // }
    }

    #[tokio::test]
    async fn test_convert_ts_with_pixel_data_file_not_found() {
        let non_existent_file = "non_existent.dcm";
        let output_file = "output.dcm";

        let result = convert_ts_with_pixel_data(non_existent_file, 100, output_file,false).await;

        // 验证返回了FileReadError
        match result {
            Err(ChangeStatus::FileReadError(_)) => {
                // 正确返回了文件读取错误
            }
            _ => {
                panic!("Expected FileReadError, but got {:?}", result);
            }
        }

        // 确保输出文件没有被创建
        assert!(!Path::new(output_file).exists());
    }
}
