use log::{Level, Log, Metadata, Record};
use std::sync::{Arc, Mutex};

pub struct VectorLogger {
    logs: Arc<Mutex<Vec<String>>>,
}

impl VectorLogger {
    // Create a new instance
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Retrieve all logs
    pub fn get_logs(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }
}

impl Log for VectorLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info // Log only up to Info level; adjust as needed
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
