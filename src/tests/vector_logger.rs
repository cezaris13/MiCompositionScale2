use log::{Level, LevelFilter, Log, Metadata, Record};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub struct VectorLogger {
    logs: Arc<Mutex<Vec<String>>>,
}

impl VectorLogger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_logs(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }

    pub fn clear_logs(&self) {
        let mut logs = self.logs.lock().unwrap();
        logs.clear();
    }
}

impl Log for VectorLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut logs = self.logs.lock().unwrap();
            logs.push(format!("[{}] {}", record.level(), record.args()));
        }
    }

    fn flush(&self) {
        // No-op for this simple implementation
    }
}

pub static LOGGER: Lazy<Arc<VectorLogger>> = Lazy::new(|| {
    let logger = VectorLogger::new();
    let logger_ref = Arc::new(logger);
    log::set_boxed_logger(Box::new(logger_ref.clone())).unwrap();
    log::set_max_level(LevelFilter::Info);
    logger_ref
});
