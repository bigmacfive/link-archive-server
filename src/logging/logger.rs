use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static LOG_BUFFER: Lazy<Mutex<Vec<LogEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub metadata: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String, module: String, metadata: Option<serde_json::Value>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp,
            level,
            message,
            module,
            metadata,
        }
    }
}

pub fn log(level: LogLevel, message: String, module: String, metadata: Option<serde_json::Value>) {
    let entry = LogEntry::new(level, message, module, metadata);
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        buffer.push(entry);
    }
}

pub fn debug(message: String, module: String, metadata: Option<serde_json::Value>) {
    log(LogLevel::Debug, message, module, metadata);
}

pub fn info(message: String, module: String, metadata: Option<serde_json::Value>) {
    log(LogLevel::Info, message, module, metadata);
}

pub fn warning(message: String, module: String, metadata: Option<serde_json::Value>) {
    log(LogLevel::Warning, message, module, metadata);
}

pub fn error(message: String, module: String, metadata: Option<serde_json::Value>) {
    log(LogLevel::Error, message, module, metadata);
}

pub fn get_logs() -> Vec<LogEntry> {
    LOG_BUFFER.lock()
        .map(|buffer| buffer.clone())
        .unwrap_or_default()
}

pub fn clear_logs() {
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        buffer.clear();
    }
}

pub fn get_logs_by_level(level: LogLevel) -> Vec<LogEntry> {
    LOG_BUFFER.lock()
        .map(|buffer| {
            buffer.iter()
                .filter(|entry| std::mem::discriminant(&entry.level) == std::mem::discriminant(&level))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_logs_by_module(module: &str) -> Vec<LogEntry> {
    LOG_BUFFER.lock()
        .map(|buffer| {
            buffer.iter()
                .filter(|entry| entry.module == module)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}