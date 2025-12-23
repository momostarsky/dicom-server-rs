use dicom_core::{DicomValue, PrimitiveValue};
use dicom_dictionary_std::tags;
use dicom_object;
use dicom_object::OpenFileOptions;
use dicom_pixeldata::Transcode;
use dicom_transfer_syntax_registry::entries::DEFLATED_EXPLICIT_VR_LITTLE_ENDIAN;
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
pub async fn convert_ts_with_gdcm_conv(
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

pub async fn convert_ts_with_transcode(
    src_file: &str,
    output_path: &str,
    overwrite: bool,
    xpatient_id: Option<&str>,
) -> Result<(), ChangeStatus> {
    let mut obj = match OpenFileOptions::new().open_file(src_file) {
        Ok(obj) => obj,
        Err(e) => {
            return Err(ChangeStatus::FileReadError(format!(
                "Failed to open file {}: {}",
                src_file, e
            )));
        }
    };
    if obj.get(tags::PIXEL_DATA).is_none(){
        return Ok(());
    }

    // transcode to explicit VR little endian
    obj.transcode(&DEFLATED_EXPLICIT_VR_LITTLE_ENDIAN.erased())
        .expect("Should have transcoded successfully");

    // check transfer syntax
    assert_eq!(
        obj.meta().transfer_syntax(),
        DEFLATED_EXPLICIT_VR_LITTLE_ENDIAN.uid()
    );

    if let Some(patient_id) = xpatient_id {

        obj.update_value(tags::PATIENT_ID, move |value| {
            *value =  DicomValue::Primitive(PrimitiveValue::Str(patient_id.into()));
        });
    }
    match obj.write_to_file(output_path) {
        Ok(_) => {}
        Err(e) => {
            return Err(ChangeStatus::FileWriteError(format!(
                "Failed to write to file {}: {}",
                output_path, e
            )));
        }
    }
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
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("./data/DeflatedExplicitVRLittleEndian.dcm", "./data/x-0.dcm" ,"P0000")]
    #[case("./data/ExplicitVRBigEndian.dcm", "./data/x-1.dcm", "P0001")]
    #[case("./data/ExplicitVRLittleEndian.dcm", "./data/x-2.dcm", "P0002")]
    #[case("./data/ImplicitVRLittleEndian.dcm", "./data/x-3.dcm", "P0003")]
    #[case("./data/JPEG2000Lossless.dcm", "./data/x-4.dcm", "P0004")]
    #[case("./data/JPEG2000Lossy.dcm", "./data/x-5.dcm", "P0005")]
    #[case("./data/JPEGProcess1.dcm", "./data/x-6.dcm", "P0006")]
    #[case("./data/JPEGProcess2_4.dcm", "./data/x-7.dcm", "P0007")]
    #[case("./data/RLELossless.dcm", "./data/x-8.dcm", "P0008")]
    #[tokio::test]
    async fn test_change_file_transfer_success(#[case] input: &str, #[case] output: &str, #[case] patient_id: &str) {
        println!("input: {}, output: {}", input, output);
        println!("PWD:{}", env!("PWD"));

        let result = convert_ts_with_transcode(input, output, false, Some(patient_id)).await;
        assert!(result.is_ok());
    }

    #[rstest]
    #[case(
        "./data/DeflatedExplicitVRLittleEndian.dcm",
        "./data/y-0.dcm"
    )]
    #[case("./data/ExplicitVRBigEndian.dcm", "./data/y-1.dcm")]
    #[case("./data/ExplicitVRLittleEndian.dcm", "./data/y-2.dcm")]
    #[case("./data/ImplicitVRLittleEndian.dcm", "./data/y-3.dcm")]
    #[case("./data/JPEG2000Lossless.dcm", "./data/y-4.dcm")]
    #[case("./data/JPEG2000Lossy.dcm", "./data/y-5.dcm")]
    #[case("./data/JPEGProcess1.dcm", "./data/y-6.dcm")]
    #[case("./data/JPEGProcess2_4.dcm", "./data/y-7.dcm")]
    #[case("./data/RLELossless.dcm", "./data/y-8.dcm")]
    #[tokio::test]
    async fn test_convert_ts_with_gdcm_conv(#[case] input: &str, #[case] output: &str) {
        println!("input: {}, output: {}", input, output);
        println!("PWD:{}", env!("PWD"));
        // 获取文件大小
        let metadata = fs::metadata(input).unwrap();
        let file_size = metadata.len() as usize;
        let result = convert_ts_with_gdcm_conv(input, file_size, output, false).await;
        assert!(result.is_ok());

    }
}
