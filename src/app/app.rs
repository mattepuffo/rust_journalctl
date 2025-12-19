use crate::load_journalctl_logs;
use crate::models::log_entry::LogEntry;
use crate::models::message::Message;
use iced::widget::{Column, button, container, row, scrollable, text, text_input};
use iced::{Application, Element, Length, Renderer, Task, Theme};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JournalEntry {
    #[serde(rename = "MESSAGE")]
    pub message: Option<String>,
    #[serde(rename = "_SYSTEMD_UNIT")]
    pub unit: Option<String>,
    #[serde(rename = "PRIORITY")]
    pub priority: Option<String>,
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    pub timestamp: Option<String>,
    #[serde(rename = "SYSLOG_IDENTIFIER")]
    pub syslog_id: Option<String>,
}

pub struct JournalApp {
    pub logs: Vec<LogEntry>,
    pub filtered_logs: Vec<LogEntry>,
    pub filter: String,
    pub line_count: String,
    pub loading: bool,
    pub error_message: Option<String>,
}

impl JournalApp {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadLogs => {
                self.loading = true;
                self.error_message = None;
                let line_count = self.line_count.clone();

                Task::perform(
                    async move { load_journalctl_logs(&line_count).await },
                    Message::LogsLoaded,
                )
            }
            Message::LogsLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(logs) => {
                        self.logs = logs;
                        self.apply_filter();
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::UpdateFilter(filter) => {
                self.filter = filter;
                self.apply_filter();
                Task::none()
            }
            Message::UpdateLineCount(count) => {
                self.line_count = count;
                Task::none()
            }
            Message::ClearFilter => {
                self.filter.clear();
                self.apply_filter();
                Task::none()
            }
        }
    }

    pub fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_logs = self.logs.clone();
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.filtered_logs = self
                .logs
                .iter()
                .filter(|log| {
                    log.message.to_lowercase().contains(&filter_lower)
                        || log.unit.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect();
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text("Journalctl Viewer").size(32).width(Length::Fill);

        let controls = row![
            text("Righe:").size(16),
            text_input("100", &self.line_count)
                .on_input(Message::UpdateLineCount)
                .width(80),
            button("Carica Log").on_press(Message::LoadLogs),
            text_input("Filtra...", &self.filter)
                .on_input(Message::UpdateFilter)
                .width(250),
            button("Cancella").on_press(Message::ClearFilter),
        ]
        .spacing(10)
        .padding(10);

        let status = if self.loading {
            text("Caricamento...").size(14)
        } else if let Some(ref error) = self.error_message {
            text(format!("Errore: {}", error)).size(14)
        } else {
            text(format!(
                "Visualizzati {} log di {}",
                self.filtered_logs.len(),
                self.logs.len()
            ))
            .size(14)
        };

        let mut log_list = Column::new().spacing(2).padding(10);

        for (idx, log) in self.filtered_logs.iter().enumerate() {
            let priority_color = match log.priority.as_str() {
                "0" | "1" | "2" | "3" => iced::Color::from_rgb(0.9, 0.2, 0.2), // Rosso
                "4" => iced::Color::from_rgb(0.9, 0.6, 0.0),                   // Arancione
                "5" => iced::Color::from_rgb(0.2, 0.6, 0.9),                   // Blu
                _ => iced::Color::from_rgb(0.5, 0.5, 0.5),                     // Grigio
            };

            let log_row = row![
                text(format!("{:3}.", idx + 1)).size(12),
                text(&log.priority_text)
                    .size(12)
                    .width(60)
                    .color(priority_color),
                text(&log.unit).size(12).width(200),
                text(&log.message).size(12),
            ]
            .spacing(10);

            log_list = log_list.push(log_row);
        }

        let logs_scroll = scrollable(log_list).height(Length::Fill);

        let content = iced::widget::column![title, controls, status, logs_scroll]
            .spacing(10)
            .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }
}

impl Default for JournalApp {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            filtered_logs: Vec::new(),
            filter: String::new(),
            line_count: "100".to_string(),
            loading: false,
            error_message: None,
        }
    }
}
