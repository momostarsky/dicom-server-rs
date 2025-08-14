pub async fn get_dicom_files_in_dir(p0: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = std::path::Path::new(p0);

    if path.is_file() {
        // 如果是单个文件，直接检查是否为DICOM文件
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("dcm") {
                return Ok(vec![p0.to_string()]);
            }
        }
        return Ok(vec![]);
    }

    // 如果是目录，则递归查找
    let mut dicom_files = Vec::new();
    collect_dicom_files(p0, &mut dicom_files)?;
    Ok(dicom_files)
}

// 辅助函数：递归收集DICOM文件
pub fn collect_dicom_files(
    dir_path: &str,
    dicom_files: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取目录项
    let entries = std::fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // 如果是目录，递归处理
            collect_dicom_files(&path.to_string_lossy(), dicom_files)?;
        } else if path.is_file() {
            // 如果是文件，检查扩展名
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("dcm") {
                    // 收集DICOM文件路径
                    dicom_files.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    Ok(())
}


// pub fn setup_logging() {
//     use tracing_subscriber::{EnvFilter, fmt, prelude::*};
//     use tracing_appender::rolling::{RollingFileAppender, Rotation};
//     use std::io::stdout;
//
//     // 创建日志文件appender，每天滚动一次
//     let file_appender = RollingFileAppender::new(
//         Rotation::DAILY,
//         "./logs", // 日志文件目录
//         "consumer.log", // 日志文件名前缀
//     );
//
//     // 创建控制台appender
//     let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);
//     let (non_blocking_stdout, _guard2) = tracing_appender::non_blocking(stdout());
//
//     // 构建日志订阅者
//     tracing_subscriber::registry()
//         .with(EnvFilter::from_default_env())
//         .with(
//             fmt::layer()
//                 .with_writer(non_blocking_file.with_max_level(tracing::Level::INFO)) // 文件日志记录INFO及以上级别
//                 .with_ansi(false) // 文件日志不使用ANSI颜色
//         )
//         .with(
//             fmt::layer()
//                 .with_writer(non_blocking_stdout.with_max_level(tracing::Level::DEBUG)) // 控制台日志记录DEBUG及以上级别
//                 .with_ansi(true) // 控制台日志使用ANSI颜色
//         )
//         .init();
//
//     // 将guard存储在全局变量中以防止被释放
//     std::mem::forget(_guard);
//     std::mem::forget(_guard2);
// }

pub fn setup_logging(policy_name: & str) {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::append::rolling_file::RollingFileAppender;
    use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
    use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
    use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Root, Logger};
    use std::path::PathBuf;

    // 创建控制台appender
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} [{M}] {m}{n}",
        )))
        .build();

    // 创建基于文件大小的触发器 (例如: 5MB)
    let trigger = Box::new(SizeTrigger::new(5 * 1024 * 1024)); // 10MB

    // 创建删除旧文件的roller (保留最多7个文件，相当于最近7天)
    let roller = Box::new(DeleteRoller::new());

    // 创建复合策略
    let policy = Box::new(CompoundPolicy::new(trigger, roller));

    // 创建滚动文件appender
    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} [{M}] {m}{n}",
        )))
        .build( format!("./logs/{}.log", policy_name), policy)
        .unwrap();

    // 构建配置
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("app", LevelFilter::Debug))
        .build(Root::builder()
            .appender("stdout")
            .appender("logfile")
            .build(LevelFilter::Info))
        .unwrap();

    // 初始化log4rs
    let _handle = log4rs::init_config(config).unwrap();
}
