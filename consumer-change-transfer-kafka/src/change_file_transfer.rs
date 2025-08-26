use dicom_object;
use dicom_object::open_file;
use gdcm_conv::PhotometricInterpretation;
use gdcm_conv::TransferSyntax as ts;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub enum ChangeStatus {
    Success,
    FileReadError(String),
    FileWriteError(String),
    ConversionError(String),
    OtherError(String),
}

impl std::fmt::Display for ChangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeStatus::Success => write!(f, "Success"),
            ChangeStatus::FileReadError(msg) => write!(f, "File read error: {}", msg),
            ChangeStatus::FileWriteError(msg) => write!(f, "File write error: {}", msg),
            ChangeStatus::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            ChangeStatus::OtherError(msg) => write!(f, "Other error: {}", msg),
        }
    }
}
impl std::error::Error for ChangeStatus {}

pub fn convert_ts_with_pixel_data(
    src_file: &str,
    file_size: usize,
    output_path: &str,
) -> Result<(), ChangeStatus> {
    // 步骤 1: 读取 DICOM 文件
    let obj = match open_file(src_file) {
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
                Ok(_) => match fs::copy(output_path, src_file) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(ChangeStatus::FileWriteError(format!(
                            "Failed to copy  file from  {} to  {}",
                            output_path, src_file
                        )));
                    }
                },
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
