#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub unit: String,
    pub priority: String,
    pub priority_text: String,
    pub timestamp: String,
}