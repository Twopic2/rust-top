use ratatui::{
    layout::Rect,
    Frame,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

use crate::data::temp::TempData;


#[derive(Default)]
pub struct TempWidget {
    cpu_name:   Option<String>,
    disk_name:  Option<String>,
    nic_name:   Option<String>,
    line_count: u8,
}

impl TempWidget {
    pub fn filter(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(all_temps) = TempData::all_temps() {
            let mut has_cpu = false;
            let mut has_disk = false;

            for (label, temp_opt) in all_temps {
                if let Some(_temp) = temp_opt {
                    if !has_cpu && label.contains("tdie") {
                        self.cpu_name = Some(label);
                        has_cpu = true;
                        self.line_count += 1;
                    } else if !has_disk && label.contains("NAND") {
                        self.disk_name = Some(label);
                        has_disk = true;
                        self.line_count += 1;
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        if let Some(all_temps) = TempData::all_temps() {
            for (label, _temp_out) in all_temps {
                if self.cpu_name.is_none()
                    && (label.contains("coretemp") || label.contains("Package id")
                        || label.contains("k10temp") || label.contains("Tdie"))
                {
                    self.cpu_name = Some(label);
                    self.line_count += 1;
                } else if self.disk_name.is_none() && label.contains("nvme") {
                    self.disk_name = Some(label);
                    self.line_count += 1;
                } else if self.nic_name.is_none() && label.contains("iwlwifi") {
                    self.nic_name = Some(label);
                    self.line_count += 1;
                }
            }
        }
    }

    pub fn get_height(&self) -> u16 {
        (self.line_count as u16).max(1) + 2
    }

    pub fn get_length(&self) -> u16 {
        let max = [&self.cpu_name, &self.disk_name, &self.nic_name]
            .iter()
            .filter_map(|n| n.as_ref())
            .map(|n| n.len() + 10)
            .max()
            .unwrap_or(16);

        (max as u16 + 4).max(20)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if let Some(all_temps) = TempData::all_temps() {
            let mut lines: Vec<Line> = Vec::new();

            for (label, temp_opt) in &all_temps {
                if let Some(temp) = temp_opt {
                    let is_shown = self.cpu_name.as_deref() == Some(label.as_str())
                        || self.disk_name.as_deref() == Some(label.as_str())
                        || self.nic_name.as_deref() == Some(label.as_str());

                    if is_shown {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{}: ", label),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{:.1}°C", temp),
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                    }
                }
            }

            let block = Block::default()
                .borders(Borders::ALL)
                .title("Temperatures")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

            frame.render_widget(Paragraph::new(lines).block(block), area);
        }
    }
}
