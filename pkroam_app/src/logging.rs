use log::LevelFilter;
use std::io;
use std::path::Path;

pub fn initialize(enable_debug: bool, log_dir: impl AsRef<Path>) -> io::Result<()> {
    let log_level = if enable_debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    const MAX_LOG_FILE_INDEX: u64 = 2;

    (MAX_LOG_FILE_INDEX..=0)
        .map(|idx| {
            let mut log_file_path = log_dir.as_ref().to_path_buf();
            let mut next_log_file_path = log_dir.as_ref().to_path_buf();
            log_file_path.push(format!("pkroam.log.{idx}"));
            next_log_file_path.push(format!("pkroam.log.{}", idx + 1));
            if log_file_path.exists() {
                std::fs::copy(&log_file_path, &next_log_file_path)?;
                std::fs::remove_file(&log_file_path)?;
            }
            Ok(())
        })
        .collect::<io::Result<Vec<_>>>()?;

    let mut log_file_path = log_dir.as_ref().to_path_buf();
    log_file_path.push(format!("pkroam.log.{}", MAX_LOG_FILE_INDEX + 1));
    let _ = std::fs::remove_file(&log_file_path);

    let mut current_log_file_path = log_dir.as_ref().to_path_buf();
    current_log_file_path.push(format!("pkroam.log"));
    let mut last_log_file_path = log_dir.as_ref().to_path_buf();
    last_log_file_path.push(format!("pkroam.log.0"));
    std::fs::copy(&current_log_file_path, last_log_file_path)?;

    std::fs::File::create(&current_log_file_path)?;
    simple_logging::log_to_file(&current_log_file_path, log_level)
}
