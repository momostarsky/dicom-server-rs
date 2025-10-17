use dicom_core::chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::string_ext::BoundedString;

// patient.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientEntity {
    pub tenant_id: String,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub patient_sex: Option<String>,
    pub patient_birth_date: Option<chrono::NaiveDate>,
    pub patient_birth_time: Option<chrono::NaiveTime>,
    pub ethnic_group: Option<String>,
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

// study.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntity {
    pub tenant_id: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub patient_age: Option<String>,
    pub patient_size: Option<f64>,
    pub patient_weight: Option<f64>,
    pub medical_alerts: Option<String>,
    pub allergies: Option<String>,
    pub pregnancy_status: Option<i32>,
    pub occupation: Option<String>,
    pub additional_patient_history: Option<String>,
    pub patient_comments: Option<String>,
    pub study_date: chrono::NaiveDate,
    pub study_time: Option<chrono::NaiveTime>,
    pub accession_number: Option<String>,
    pub study_id: Option<String>,
    pub study_description: Option<String>,
    pub referring_physician_name: Option<String>,
    pub admission_id: Option<String>,
    pub performing_physician_name: Option<String>,
    pub procedure_code_sequence: Option<String>,
    pub received_instances: Option<i32>,
    pub space_size: Option<i64>,
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
    pub study_date_origin: String,       // 新增字段
   
}

// series.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesEntity {
    pub tenant_id: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub modality: String,
    pub series_number: Option<i32>,
    pub series_date: Option<chrono::NaiveDate>,
    pub series_time: Option<chrono::NaiveTime>,
    pub series_description: Option<String>,
    pub body_part_examined: Option<String>,
    pub protocol_name: Option<String>,
    pub acquisition_number: Option<i32>,
    pub acquisition_time: Option<chrono::NaiveTime>,
    pub acquisition_date: Option<chrono::NaiveDate>,
    pub acquisition_date_time: Option<NaiveDateTime>,
    pub performing_physician_name: Option<String>,
    pub operators_name: Option<String>,
    pub number_of_series_related_instances: Option<i32>,
    pub received_instances: Option<i32>, // 新增字段
    pub space_size: Option<i64>,         // 新增字段
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
   

}

// image.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntity {
    pub tenant_id: String,
    pub sop_instance_uid: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub instance_number: Option<i32>,
    pub image_comments: Option<String>,
    pub content_date: Option<chrono::NaiveDate>,
    pub content_time: Option<chrono::NaiveTime>,
    pub acquisition_date: Option<chrono::NaiveDate>,
    pub acquisition_time: Option<chrono::NaiveTime>,
    pub acquisition_date_time: Option<chrono::NaiveDateTime>,
    pub image_type: Option<String>,
    pub image_orientation_patient: Option<String>,
    pub image_position_patient: Option<String>,
    pub slice_thickness: Option<f64>,
    pub spacing_between_slices: Option<f64>,
    pub slice_location: Option<f64>,
    pub samples_per_pixel: Option<i32>,
    pub photometric_interpretation: Option<String>,
    pub width: Option<i32>,
    pub columns: Option<i32>,
    pub bits_allocated: Option<i32>,
    pub bits_stored: Option<i32>,
    pub high_bit: Option<i32>,
    pub pixel_representation: Option<i32>,
    pub rescale_intercept: Option<f64>,
    pub rescale_slope: Option<f64>,
    pub rescale_type: Option<String>,
    pub window_center:Option<String>,
    pub window_width:  Option<String>,
    pub number_of_frames: i32,


    /*
    常见的应用场景
        图像重建算法：例如，区分使用了“滤波反投影 (Filtered Back Projection)”还是“迭代重建 (Iterative Reconstruction)”算法。
        后处理滤波：标识应用了哪些空间滤波器，如“锐化 (Edge Enhancement)”、“平滑 (Smoothing)”或“降噪 (Noise Reduction)”。
        特殊成像模式：用于标识特定的采集或处理模式，如“能谱成像处理”、“去金属伪影处理 (Metal Artifact Reduction)”等。
        数据校正：表示进行了哪些校正，如“散射校正”、“衰减校正”等。
    为什么需要它？
        互操作性：不同制造商的设备可能用不同的术语描述相似的处理。标准化的代码确保了信息在不同系统（如 PACS, RIS, 工作站）之间交换时的准确理解。
        自动化处理：下游系统（如 AI 分析工具、图像分析软件）可以根据这个代码来判断图像的处理状态，从而调整其分析算法或解释结果。例如，知道图像经过了强烈的锐化处理，可能会影响对边缘或纹理的分析。
        研究与质量保证：研究人员可以利用这些代码来筛选特定处理方式的图像集。质量保证流程可以检查预期的处理代码是否被正确应用。
     */
    // 设备处理描述  一个机器可读的、标准化的代码，确保不同厂商和系统之间对处理步骤的理解一致
    pub acquisition_device_processing_description: Option<String>,
    // 设备处理代码  一个人类可读的文本描述（如 "Edge Enhancement", "Noise Reduction", "Filtered Back Projection"）
    pub acquisition_device_processing_code: Option<String>,
    /*
    核心功能
    唯一标识设备：这是识别执行医学影像采集或处理的物理设备（如 CT 扫描仪、MRI 机器、X 光机、超声设备、工作站等）的主要方式之一。它通常是由设备制造商分配的、在该制造商产品线中唯一的序列号。
    设备溯源：当需要追踪图像来源、进行质量控制、故障排查、维护记录查询或法规审计时，设备序列号是至关重要的信息。它能精确地定位到生成特定图像的那台具体机器。
    数据关联：在 PACS（影像归档与通信系统）、RIS（放射信息系统）或研究数据库中，可以根据设备序列号来筛选、统计或分析来自特定设备的所有影像数据。
     */
    pub device_serial_number: Option<String>,
    pub software_versions: Option<String>,
    pub transfer_syntax_uid: String,
    pub pixel_data_location: Option<String>,
    pub thumbnail_location: Option<String>,
    pub sop_class_uid: String,
    pub image_status: Option<String>,
    pub space_size: Option<u64>, // 新增字段
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}
