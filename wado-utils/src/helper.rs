use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn get_current_time() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

#[allow(dead_code)]
/// 递归遍历目录，并收集所有.dcm文件
pub fn collect_dicom_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归遍历子目录
                collect_dicom_files(&path, files);
            } else if path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("dcm"))
            {
                // 添加.dcm文件到列表
                files.push(path);
            }
        }
    }
}
#[allow(dead_code)]
/// 递归遍历目录，并收集所有指定文件类型 例如: JPG, PNG, GIF
pub fn collect_dicom_files_withext(
    dir: &Path,
    file_extentions: &str,
    files: &mut Vec<std::path::PathBuf>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归遍历子目录
                collect_dicom_files(&path, files);
            } else if path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case(file_extentions))
            {
                // 添加.dcm文件到列表
                files.push(path);
            }
        }
    }
}
