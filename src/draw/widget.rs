use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    Frame,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

use crate::unix::temp::TempData;

pub struct TempWidget {
    temp_data: TempData,
}

impl TempWidget {
    pub fn new() -> Self {
        Self {
            temp_data: TempData::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(all_temps) = self.temp_data.get_all_temps() {
            let mut total_temp: HashMap<String, f32> = HashMap::new();
            let mut has_tdie = false;

            for (label, temp_opt) in all_temps {
                /* For apple products */
                if label.contains("tdie") {
                    if !has_tdie {
                        if let Some(temp) = temp_opt {
                            total_temp.insert(label, temp);
                            has_tdie = true;
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
            let mut temp_vec: Vec<(&String, &f32)> = total_temp.iter().collect();
            temp_vec.sort_by(|a, b| a.0.cmp(b.0));

            let max_lines = area.height.saturating_sub(2) as usize;

            for (label, temp) in temp_vec.iter().take(max_lines) {
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