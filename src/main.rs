mod app;
mod models;

use crate::app::app::{JournalApp, JournalEntry};
use crate::models::log_entry::LogEntry;
use iced::{Size, window};
use rdev::display_size;
use std::process::Command;

fn main() -> iced::Result {
    let (w, h) = display_size().unwrap();

    let window_width = w as f32 * 0.8;
    let window_height = h as f32 * 0.8;

    let window_settings = window::Settings {
        size: Size::new(window_width, window_height),
        ..Default::default()
    };

    iced::application(JournalApp::new(), JournalApp::update(), JournalApp::view())
        .theme(JournalApp::theme())
        .run()

    // iced::run(JournalApp::update, JournalApp::view)
}

pub async fn load_journalctl_logs(line_count: &str) -> Result<Vec<LogEntry>, String> {
    let count = line_count.parse::<u32>().unwrap_or(100);

    let output = Command::new("journalctl")
        .arg("-n")
        .arg(count.to_string())
        .arg("--no-pager")
        .arg("-o")
        .arg("json")
        .arg(
            "--output-fields=MESSAGE,_SYSTEMD_UNIT,PRIORITY,__REALTIME_TIMESTAMP,SYSLOG_IDENTIFIER",
        )
        .output()
        .map_err(|e| format!("Errore esecuzione journalctl: {}. Prova con sudo.", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("journalctl fallito: {}", error));
    }

    let logs_str = String::from_utf8_lossy(&output.stdout);
    let mut logs = Vec::new();

    for line in logs_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<JournalEntry>(line) {
            Ok(entry) => {
                let priority = entry.priority.as_deref().unwrap_or("6").to_string();
                let priority_text = match priority.as_str() {
                    "0" => "EMERG",
                    "1" => "ALERT",
                    "2" => "CRIT",
                    "3" => "ERROR",
                    "4" => "WARN",
                    "5" => "NOTICE",
                    "6" => "INFO",
                    "7" => "DEBUG",
                    _ => "?",
                }
                .to_string();

                logs.push(LogEntry {
                    message: entry.message.unwrap_or_else(|| "(no message)".to_string()),
                    unit: entry
                        .unit
                        .or(entry.syslog_id)
                        .unwrap_or_else(|| "unknown".to_string()),
                    priority,
                    priority_text,
                    timestamp: entry.timestamp.unwrap_or_default(),
                });
            }
            Err(e) => {
                eprintln!("Errore parsing: {}", e);
            }
        }
    }

    Ok(logs)
}
