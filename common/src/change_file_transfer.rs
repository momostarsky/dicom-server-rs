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

//修改传输语法为 RLELossless
// 建议:采用FO_DICOM库提供的转换接口方式, 可以通过gRPC模式调用
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
        gdcm_conv::TransferSyntax::ExplicitVRLittleEndian,
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
    use rstest::rstest;
    use std::fs;

    #[rstest]
    // #[case(
    //     "./data/DeflatedExplicitVRLittleEndian.dcm",
    //     "./data/x-DeflatedExplicitVRLittleEndian.dcm"
    // )]
    #[case("./data/ExplicitVRBigEndian.dcm", "./data/x-ExplicitVRBigEndian.dcm")]
    #[case(
        "./data/ExplicitVRLittleEndian.dcm",
        "./data/x-ExplicitVRLittleEndian.dcm"
    )]
    #[case(
        "./data/ImplicitVRLittleEndian.dcm",
        "./data/x-ImplicitVRLittleEndian.dcm"
    )]
    #[case("./data/JPEG2000Lossless.dcm", "./data/x-JPEG2000Lossless.dcm")]
    #[case("./data/JPEG2000Lossy.dcm", "./data/x-JPEG2000Lossy.dcm")]
    #[case("./data/JPEGProcess1.dcm", "./data/x-JPEGProcess1.dcm")]
    #[case("./data/JPEGProcess2_4.dcm", "./data/x-JPEGProcess2_4.dcm")]
    #[case("./data/RLELossless.dcm", "./data/x-RLELossless.dcm")]
    #[tokio::test]
    async fn test_change_file_transfer_success(#[case] input: &str, #[case] output: &str) {
        println!("input: {}, output: {}", input, output);
        println!("PWD:{}", env!("PWD"));
        // 获取文件大小
        let metadata = fs::metadata(input).unwrap();
        let file_size = metadata.len() as usize;
        let result = convert_ts_with_pixel_data(input, file_size, output, false).await;
        assert!(result.is_ok());
    }
}
