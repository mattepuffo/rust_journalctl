use crate::models::boot_info::BootInfo;
use crate::models::log_entry::LogEntry;

#[derive(Debug, Clone)]
pub enum Message {
    LoadLogs,
    LogsLoaded(Result<Vec<LogEntry>, String>),
    UpdateFilter(String),
    UpdateLineCount(String),
    ClearFilter,
    ShowCurrentBoot,
    ShowBootList,
    BootListLoaded(Result<Vec<BootInfo>, String>),
    SelectBoot(i32),
    Export,
}
