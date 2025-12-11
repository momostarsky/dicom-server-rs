use dicom_core::Tag;
use std::fs;
use std::path::PathBuf;

use dicom_object::{DefaultDicomObject, FileDicomObject, FileMetaTableBuilder, OpenFileOptions};

use crate::dicom_utils::get_tag_values;
use crate::storage_config::StorageConfig;
use crate::{dicom_utils, server_config};
use database::dicom_meta::DicomStateMeta;
use dicom_dictionary_std::tags;
use dicom_object::file::CharacterSetOverride;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Write};
use tokio::task;

pub fn file_exists(file_path: &PathBuf) -> bool {
    fs::metadata(file_path).is_ok()
}
/// 递归遍历目录下的所有文件
pub fn walk_directory<P: Into<PathBuf>>(start_path: P) -> Result<Vec<PathBuf>, Error> {
    let start_path = start_path.into();
    let mut file_paths = Vec::new();

    if start_path.is_dir() {
        for entry in fs::read_dir(start_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 递归处理子目录
                file_paths.extend(walk_directory(path)?);
            } else {
                // 收集文件路径
                file_paths.push(path);
            }
        }
    } else if start_path.is_file() {
        // 如果是单个文件，则直接添加
        file_paths.push(start_path);
    }

    Ok(file_paths)
}
pub fn get_string(tag: Tag, dicom_obj: &DefaultDicomObject) -> String {
    dicom_utils::get_text_value(dicom_obj, tag).unwrap_or_else(|| String::from(""))
}

pub fn generate_study_json(
    file: &PathBuf,
    json_save_to: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if !file_exists(file) {
        eprintln!("File does not exist: {:?}", file);
        return Err(Box::new(Error::new(
            std::io::ErrorKind::NotFound,
            format!("File or Directory does not exist: {:?}", file),
        )));
    }
    if !file.is_dir() {
        // 递归遍历目录下的所有文件
        eprintln!("File does not exist: {:?}", file);
        return Err(Box::new(Error::new(
            std::io::ErrorKind::NotFound,
            format!("File or Directory does not exist: {:?}", file),
        )));
    }
    let files = walk_directory(file)?;
    if files.is_empty() {
        eprintln!("No DICOM files found in the directory: {:?}", file);
        return Ok(());
    }
    let media_storage_sop_instance_uid = "DHZ.1.2.25.280986007.1.65029756031778";
    let empty_meta = FileMetaTableBuilder::new()
        .transfer_syntax(dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.uid())
        .media_storage_sop_class_uid("1.2.840.10008.5.1.4.1.1.1")
        .media_storage_sop_instance_uid(media_storage_sop_instance_uid)
        .implementation_class_uid("1.2.345.6.7890.1.234")
        .build()
        .unwrap();
    let results: Vec<(
        HashMap<(String, u32), Value>,
        HashMap<(String, String, u32), Value>,
    )> = files
        .par_iter()
        .map(|file| {
            let mut local_seris_map = HashMap::new();
            let mut local_sop_map = HashMap::new();
            let obj = OpenFileOptions::new()
                .charset_override(CharacterSetOverride::AnyVr)
                .read_until(tags::PIXEL_DATA)
                .open_file(&file)
                .unwrap_or_else(|_| FileDicomObject::new_empty_with_meta(empty_meta.clone()));

            if get_string(tags::MEDIA_STORAGE_SOP_INSTANCE_UID, &obj)
                != media_storage_sop_instance_uid
            {
                let series_uid = get_string(tags::SERIES_INSTANCE_UID, &obj);
                // 只有当series_uid非空时才处理
                if !series_uid.is_empty() {
                    let sn = get_string(tags::SERIES_NUMBER, &obj);
                    let series_num = sn.parse::<u32>().unwrap_or(0);
                    if !local_seris_map.contains_key(&(series_uid.clone(), series_num)) {
                        let sex = get_string(tags::PATIENT_SEX, &obj);
                        let age = get_string(tags::PATIENT_AGE, &obj);
                        let name = get_string(tags::PATIENT_NAME, &obj);
                        let paid = get_string(tags::PATIENT_ID, &obj);
                        let birth_date = get_string(tags::PATIENT_BIRTH_DATE, &obj);
                        let modality = get_string(tags::MODALITY, &obj);
                        let body_part = get_string(tags::BODY_PART_EXAMINED, &obj);
                        let study_date = get_string(tags::STUDY_DATE, &obj);
                        let study_time = get_string(tags::STUDY_TIME, &obj);
                        let acc_num = get_string(tags::ACCESSION_NUMBER, &obj);
                        let manufacturer = get_string(tags::MANUFACTURER, &obj);
                        let institution_address = get_string(tags::INSTITUTION_ADDRESS, &obj);
                        let institution_name = get_string(tags::INSTITUTION_NAME, &obj);
                        let series_json = json!({
                              "00100040": sex,
                              "00101010": age,
                              "0020000E": series_uid,
                              "00100010": name,
                              "00100020": paid,
                              "00100030": birth_date,
                              "00180015": body_part,
                              "00200011": sn ,
                              "00080020": study_date,
                              "00080030": study_time,
                              "00080050": acc_num,
                              "00080060": modality,
                              "00080070": manufacturer,
                              "00080081": institution_address,
                              "00080080": institution_name,
                        });
                        local_seris_map.insert((series_uid.clone(), series_num), series_json);
                    } else {
                        println!("Series already exists: {:?}", series_uid);
                    }

                    let series_desc = get_string(tags::SERIES_DESCRIPTION, &obj);

                    let px_spacing_vec: Vec<String> = get_tag_values(tags::PIXEL_SPACING, &obj);

                    let rows = get_string(tags::ROWS, &obj);
                    let columns = get_string(tags::COLUMNS, &obj);
                    let body_part = get_string(tags::BODY_PART_EXAMINED, &obj);

                    let image_type_vec: Vec<String> = get_tag_values(tags::IMAGE_TYPE, &obj);
                    let pixel_representation = get_string(tags::PIXEL_REPRESENTATION, &obj);
                    let patient_position = get_string(tags::PATIENT_POSITION, &obj);
                    let image_position_patient_vec: Vec<String> =
                        get_tag_values(tags::IMAGE_POSITION_PATIENT, &obj);

                    let image_orientation_patient_vec: Vec<String> =
                        get_tag_values(tags::IMAGE_ORIENTATION_PATIENT, &obj);

                    let instance_num = get_string(tags::INSTANCE_NUMBER, &obj);
                    let slice_thickness = get_string(tags::SLICE_THICKNESS, &obj);
                    let sop_uid = get_string(tags::SOP_INSTANCE_UID, &obj);
                    // 只有当sop_uid非空时才处理
                    if !sop_uid.is_empty() {
                        let inst_num = instance_num.parse::<u32>().unwrap_or(0);

                        let bits_allocated = get_string(tags::BITS_ALLOCATED, &obj);
                        let bits_stored = get_string(tags::BITS_STORED, &obj);
                        let high_bit = get_string(tags::HIGH_BIT, &obj);
                        let modality = get_string(tags::MODALITY, &obj);
                        let sop_json = json!({
                          "0008103E": series_desc ,
                          "00280030": px_spacing_vec,
                          "00280010": rows  ,
                          "00280011": columns,
                          "00180015": body_part,
                          "00080008": image_type_vec,
                          "00280103": pixel_representation,
                          "00185100": patient_position,
                          "00200032": image_position_patient_vec,
                          "00180050": slice_thickness,
                          "00200013": instance_num,
                          "00200037": image_orientation_patient_vec,
                          "00080018": sop_uid,
                          "00280100": bits_allocated,
                          "00280101": bits_stored,
                          "00280102": high_bit,
                          "00080060": modality,
                        });
                        local_sop_map.insert((series_uid, sop_uid, inst_num), sop_json);
                    }
                }
            }
            (local_seris_map, local_sop_map)
        })
        .collect();

    //结果进行合并
    let mut seris_map = HashMap::new();
    let mut sop_map = HashMap::new();
    for (local_seris, local_sop) in results {
        seris_map.extend(local_seris);
        sop_map.extend(local_sop);
    }
    let mut study_vec = Vec::new();
    // 排序后的 series (series_uid, series_num, series_json)
    let mut seris_vec: Vec<(&(String, u32), &Value)> = seris_map.iter().collect();
    seris_vec.sort_by_key(|((_, series_num), _)| *series_num);

    for ((series_uid, _series_num), series_json) in seris_vec {
        // 1. 收集、排序
        let mut sop_list: Vec<(u32, &Value)> = sop_map
            .iter()
            .filter(|((s_uid, _, _), _)| s_uid == series_uid)
            .map(|((_, _, inst_num), sop_json)| (*inst_num, sop_json))
            .collect();
        sop_list.sort_by_key(|(inst_num, _)| *inst_num);
        let sop_vec: Vec<&Value> = sop_list.into_iter().map(|(_, v)| v).collect();

        // let mut sop_vec = Vec::new();
        // for (s_uid, sop_json) in sop_map.iter() {
        //     if s_uid.0 == *series_uid {
        //         sop_vec.push(sop_json);
        //     }
        // }
        // 2. 组装 series
        let series_json = series_json.as_object().unwrap();
        let json_str = json!({
              "00100040": series_json["00100040"],
              "00101010": series_json["00101010"],
              "0020000E": series_uid,
              "00100010": series_json["00100010"],
              "00100020": series_json["00100020"],
              "00100030": series_json["00100030"],
              "00180015": series_json["00180015"],
              "00200011": series_json["00200011"] ,
              "00080020": series_json["00080020"],
              "00080030": series_json["00080030"],
              "00080050": series_json["00080050"],
              "00080060": series_json["00080060"],
              "00080070": series_json["00080070"],
              "00080081": series_json["00080081"],
              "00080080": series_json["00080080"],
              "sopData":sop_vec
        });
        study_vec.push(json_str);
    }

    let study_json = json!({
       "seriesData": study_vec,
       "hiscode":"89269",
       "expires":"2025-06-20T13-05-16",
       "token":"cbcbc2c203fe3877737c0befd6a769fa"
    });
    let file = File::create(json_save_to);
    match file {
        Ok(mut file) => {
            file.write_all(study_json.to_string().as_bytes())
                .expect("写入文件失败");
        }
        Err(error) => {
            println!("写入文件时发生错误: {}", error);
        }
    }
    Ok(())
}

pub async fn generate_series_json(series_info: &DicomStateMeta) -> Result<String, Error> {
    let app_config = match server_config::load_config() {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("server_config::load_config failed for generate: {}", e),
            ));
        }
    };
    let storage_config = StorageConfig::make_storage_config( &app_config);

    let json_file_path = match storage_config.json_metadata_path_for_series(series_info,true) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get json_file_path for generate: {}", e),
            ));
        }
    };

    let dicom_dir = match storage_config.dicom_series_dir(series_info,false) {
        Ok(vv) => vv,
        Err(_) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Failed to retrieve dicom_dir",
            ));
        }
    };

    let files = match walk_directory(&dicom_dir) {
        Ok(files) => files,
        Err(e) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to walk directory: {}", e),
            ));
        }
    };

    if files.is_empty() {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            format!("No DICOM files found in the directory:{}", &dicom_dir),
        ));
    };

    let mut handles = vec![];

    for file_path in &files {
        // 读取 DICOM 文件内容
        let file_path_clone = file_path.clone(); // 克隆路径供异步任务使用
        let handle = task::spawn_blocking(move || {
            // 读取 DICOM 文件内容
            let sop_json = match OpenFileOptions::new()
                .charset_override(CharacterSetOverride::AnyVr)
                .read_until(tags::PIXEL_DATA)
                .open_file(&file_path_clone)
            {
                Ok(dicom_object) => {
                    let mut dicom_json = Map::new();
                    dicom_object.tags().into_iter().for_each(|tag| {
                        let value_str: Vec<String> = get_tag_values(tag, &dicom_object);
                        let vr = dicom_object.element(tag).expect("REASON").vr().to_string();
                        let tag_key = format!("{:04X}{:04X}", tag.group(), tag.element());
                        let element_json = json!({
                            "vr": vr,
                            "Value": value_str
                        });
                        dicom_json.insert(tag_key, element_json);
                    });
                    Ok(dicom_json)
                }
                Err(e) => Err(format!(
                    "Failed to read DICOM file {}: {}",
                    file_path_clone.display(),
                    e
                )),
            };
            sop_json
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let mut arr = vec![];
    for handle in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(sop_json) => arr.push(sop_json),
                Err(e) => {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to walk directory: {}", e),
                    ));
                }
            },
            Err(e) => {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to walk directory: {}", e),
                ));
            }
        }
    }

    let json = match serde_json::to_string(&arr) {
        Ok(json) => {
            if let Err(e) = fs::write(&json_file_path, &json) {
                return Err(Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("Failed to write  JSON to file: {}", e),
                ));
            }
            json
        }
        Err(e) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to generate JSON: {}", e),
            ));
        }
    };
    Ok(json)
}
