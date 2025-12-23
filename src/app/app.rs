use crate::load_journalctl_logs;
use crate::models::boot_info::BootInfo;
use crate::models::log_entry::LogEntry;
use crate::models::message::Message;
use iced::widget::{Column, Row, button, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task, Theme};
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
    pub boot_list: Vec<BootInfo>,
    pub show_boot_list: bool,
}

impl JournalApp {
    // pub fn new() -> (Self, Task<Message>) {
    //     (Self::default(), Task::none())
    // }

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
            Message::ShowCurrentBoot => {
                self.loading = true;
                self.error_message = None;

                Task::perform(
                    async move {
                        crate::load_journalctl_logs_with_args(vec![
                            "-b".to_string(),
                            "0".to_string(),
                        ])
                        .await
                    },
                    Message::LogsLoaded,
                )
            }
            Message::ShowBootList => {
                self.loading = true;
                self.error_message = None;
                self.show_boot_list = true;

                Task::perform(
                    async move { crate::load_boot_list().await },
                    Message::BootListLoaded,
                )
            }
            Message::BootListLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(boots) => {
                        self.boot_list = boots;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        self.show_boot_list = false;
                    }
                }
                Task::none()
            }
            Message::SelectBoot(boot_offset) => {
                self.loading = true;
                self.error_message = None;
                self.show_boot_list = false;

                Task::perform(
                    async move {
                        crate::load_journalctl_logs_with_args(vec![
                            "-b".to_string(),
                            boot_offset.to_string(),
                        ])
                        .await
                    },
                    Message::LogsLoaded,
                )
            }
            Message::Export => {
                // Implementa qui la logica per esportare i log
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

        let action_buttons = row![
            button("Mostra il boot log corrente").on_press(Message::ShowCurrentBoot),
            button("Mostra la lista dei boot").on_press(Message::ShowBootList),
            button("Esporta").on_press(Message::Export),
        ]
        .spacing(10)
        .padding(10);

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

        let table_header: Row<'_, Message, Theme, iced::Renderer> = row![
            text("#").size(12).width(50),
            text("Priorità").size(12).width(80),
            text("Unità").size(12).width(250),
            text("Messaggio").size(12).width(Length::Fill),
        ]
        .spacing(10)
        .padding(5);

        let header_container = container(table_header).style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.2, 0.2, 0.25,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.35),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        let mut log_list = Column::new().spacing(2).padding(10);

        for (idx, log) in self.filtered_logs.iter().enumerate() {
            let priority_color = match log.priority.as_str() {
                "0" | "1" | "2" | "3" => iced::Color::from_rgb(0.9, 0.2, 0.2),
                "4" => iced::Color::from_rgb(0.9, 0.6, 0.0),
                "5" => iced::Color::from_rgb(0.2, 0.6, 0.9),
                _ => iced::Color::from_rgb(0.5, 0.5, 0.5),
            };

            let log_row = row![
                text(format!("{}", idx + 1)).size(12).width(50),
                text(&log.priority_text)
                    .size(12)
                    .width(80)
                    .color(priority_color),
                text(&log.unit).size(12).width(250),
                text(&log.message).size(12).width(Length::Fill),
            ]
            .spacing(10)
            .padding(5);

            let row_container = container(log_row).style(move |_theme: &Theme| {
                let bg_color = if idx % 2 == 0 {
                    iced::Color::from_rgba(0.15, 0.15, 0.18, 1.0)
                } else {
                    iced::Color::from_rgba(0.12, 0.12, 0.15, 1.0)
                };

                container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.2, 0.2, 0.25),
                        width: 0.5,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            });

            log_list = log_list.push(row_container);
        }

        let logs_scroll = scrollable(log_list).height(Length::Fill);

        let content = iced::widget::column![
            title,
            action_buttons,
            controls,
            status,
            header_container,
            logs_scroll
        ]
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
            boot_list: Vec::new(),
            show_boot_list: false,
        }
    }
}
