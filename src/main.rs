mod app;
mod models;

use crate::app::app::{JournalApp, JournalEntry};
use crate::models::log_entry::LogEntry;
use iced::{Size, window};
use rdev::display_size;
use std::process::Command;
use crate::models::boot_info::BootInfo;

fn main() -> iced::Result {
    let (w, h) = display_size().unwrap();

    let window_width = w as f32 * 0.8;
    let window_height = h as f32 * 0.8;

    let window_settings = window::Settings {
        size: Size::new(window_width, window_height),
        resizable: false,
        ..Default::default()
    };

    iced::application(JournalApp::default, JournalApp::update, JournalApp::view)
        .theme(JournalApp::theme)
        .title("Journalctl Viewer")
        .window(window_settings)
        .run()
}

pub async fn load_journalctl_logs_with_args(args: Vec<String>) -> Result<Vec<LogEntry>, String> {
    let mut command = Command::new("journalctl");

    for arg in args {
        command.arg(arg);
    }

    let output = command
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

pub async fn load_journalctl_logs(line_count: &str) -> Result<Vec<LogEntry>, String> {
    let count = line_count.parse::<u32>().unwrap_or(100);
    load_journalctl_logs_with_args(vec!["-n".to_string(), count.to_string()]).await
}

pub async fn load_boot_list() -> Result<Vec<BootInfo>, String> {
    use crate::models::boot_info::BootInfo;

    let output = Command::new("journalctl")
        .arg("--list-boots")
        .arg("--no-pager")
        .output()
        .map_err(|e| format!("Errore esecuzione journalctl: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("journalctl fallito: {}", error));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut boots = Vec::new();

    for line in output_str.lines() {
        // Formato: -5 a1b2c3d4... 2024-01-15 10:23:45 CET—2024-01-15 18:45:32 CET
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let boot_offset = parts[0].parse::<i32>().unwrap_or(0);
            let boot_id = parts[1].to_string();
            let first_entry = format!("{} {}", parts[2], parts[3]);
            let last_entry = if parts.len() >= 6 {
                format!("{} {}", parts[5], parts[6])
            } else {
                "N/A".to_string()
            };

            boots.push(BootInfo {
                boot_id,
                boot_offset,
                first_entry,
                last_entry,
            });
        }
    }

    Ok(boots)
}
