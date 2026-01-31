use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    Frame,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

use crate::data::temp::TempData;

pub struct TempWidget {
    temp_data: TempData,
}

impl TempWidget {
    pub fn new() -> Self {
        Self {
            temp_data: TempData::new(),
        }
    }

    pub fn get_height(&mut self) -> u16 {
        if let Some(all_temps) = self.temp_data.get_all_temps() {
            let mut count = 0;
            let mut has_cdie = false;

            for (label, temp_opt) in all_temps {
                /* For Macos */
                if temp_opt.is_some() {
                    if label.contains("tdie") && !has_cdie {
                        count += 1;
                        has_cdie = true;
                    } else if label.contains("NAND") {
                        count += 1;
                    }
                }
                /* FOr unix */
            }

            (count as u16).max(1) + 2
        } else {
            4 
        }
    }

    pub fn get_length(&mut self) -> u16 {
        if let Some(all_temps) = self.temp_data.get_all_temps() {
            let mut max_width = 0;
            let mut has_cdie = false;

            for (label, temp_opt) in all_temps {
                if let Some(temp) = temp_opt {
                    /* For macos */
                    let should_display = if label.contains("tdie") && !has_cdie {
                        has_cdie = true;
                        true
                    } else {
                        label.contains("NAND")
                    };
                    if should_display {
                        let display_len = format!("{}: {:.1}°C", label, temp).len();
                        max_width = max_width.max(display_len);
                    }
                    /* For Unix */
                }
            }

            (max_width as u16 + 4).max(20)
        } else {
            20 
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(all_temps) = self.temp_data.get_all_temps() {
            let mut total_temp: HashMap<String, f32> = HashMap::new();
            let mut has_cpudie = false;

            for (label, temp_opt) in all_temps {
                /* For apple products */
                if label.contains("tdie") {
                    if !has_cpudie {
                        if let Some(temp) = temp_opt {
                            total_temp.insert(label, temp);
                            has_cpudie = true;
                        }
                    }
                } else if label.contains("NAND") {
                    if let Some(temp) = temp_opt {
                        total_temp.insert(label, temp);
                    }
                } 
                /* For Unix */
            }

            let mut lines: Vec<Line> = Vec::new();
            let temp_vec: Vec<(&String, &f32)> = total_temp.iter().collect();

            for (label, temp) in temp_vec.iter() {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", label), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:.1}°C", temp), Style::default().fg(Color::Green)),
                ]));
            }

            let block = Block::default()
                .borders(Borders::ALL)
                .title("Temperatures")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

            let paragraph = Paragraph::new(lines).block(block);
            frame.render_widget(paragraph, area);
        } 
    }
}
