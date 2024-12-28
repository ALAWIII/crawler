/// this file is intended to create a unified log files for success operation and failed
/// operations.
///
/// It gives you the ability to get a buffer for two log files the log_succ.txt , log_fail.txt
use once_cell::sync::OnceCell;
use std::fs::{create_dir_all, File};
use std::io::BufWriter;
use std::{env, sync::Arc};
use tokio::sync::Mutex;

type SafeWrite = Arc<Mutex<BufWriter<File>>>;
static LOG_SUCC: OnceCell<SafeWrite> = OnceCell::new();
static LOG_FAIL: OnceCell<SafeWrite> = OnceCell::new();

fn create_log_file(kind_file: &str) -> BufWriter<File> {
    let exe_path = env::current_exe().unwrap();
    let log_dir = exe_path.parent().unwrap().join("logs");
    if !log_dir.exists() {
        // creates the logs directory
        create_dir_all(&log_dir).unwrap();
    }
    let log = log_dir.join(kind_file);
    let file = File::create(log).unwrap();
    BufWriter::new(file)
}

pub fn get_log_success() -> SafeWrite {
    LOG_SUCC
        .get_or_init(|| Arc::new(Mutex::new(create_log_file("log_succ.txt"))))
        .clone()
}

pub fn get_log_failure() -> SafeWrite {
    LOG_FAIL
        .get_or_init(|| Arc::new(Mutex::new(create_log_file("log_fail.txt"))))
        .clone()
}
