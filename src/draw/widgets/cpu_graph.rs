use indexmap::IndexMap;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

use sysinfo::System;
use crate::data::info::SystemInfo;

#[derive(Clone, Copy)]
pub enum ColorScheme {
    Green,
    Cyan,
    Red,
    Yellow,
    Blue,
    Magenta,
}

impl ColorScheme {
    fn get_gradient_color(&self, percentage: f64) -> Color {
        match self {
            ColorScheme::Green => {
                if percentage < 33.0 {
                    Color::Green
                } else if percentage < 66.0 {
                    Color::LightGreen
                } else if percentage < 90.0 {
                    Color::Yellow
                } else {
                    Color::Red
                }
            }
            ColorScheme::Cyan => {
                if percentage < 50.0 {
                    Color::Cyan
                } else {
                    Color::LightCyan
                }
            }
            ColorScheme::Red => {
                if percentage < 50.0 {
                    Color::LightRed
                } else {
                    Color::Red
                }
            }
            ColorScheme::Yellow => Color::Yellow,
            ColorScheme::Blue => {
                if percentage < 50.0 {
                    Color::Blue
                } else {
                    Color::LightBlue
                }
            }
            ColorScheme::Magenta => {
                if percentage < 50.0 {
                    Color::Magenta
                } else {
                    Color::LightMagenta
                }
            }
        }
    }
}

pub struct MultiCoreGraph {
    pub cores: IndexMap<usize, f64>,
    pub color_scheme: ColorScheme,
}

impl MultiCoreGraph {
    pub fn new(num_cores: usize, color_scheme: ColorScheme) -> Self {
        let cores = (0..num_cores).map(|i| (i, 0.0)).collect();
        Self {
            cores,
            color_scheme,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, sys: &mut System) {
        let cpu_freq = SystemInfo::display_cpu_frequency(sys).unwrap_or(0);
        let cpu_freq = format!("{:.2} GHz", cpu_freq as f64 / 1000.0);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Cpu Cores")
            .title(Line::from(cpu_freq).right_aligned())
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 2 || inner_area.width < 2 {
            return;
        }

        let lines = self.generate_core_grid(inner_area.width as usize, inner_area.height as usize);
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner_area);
    }

    fn generate_core_grid(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if self.cores.is_empty() {
            return lines;
        }

        let label_width = 10;
        let min_bar_width = 10;
        let cores_per_row = (width / (label_width + min_bar_width)).max(1);
        let bar_width = (width / cores_per_row).saturating_sub(label_width);

        let all_cores: Vec<(usize, f64)> = self.cores.iter().map(|(&k, &v)| (k, v)).collect();

        for chunk in all_cores.chunks(cores_per_row) {
            if lines.len() >= height {
                break;
            }

            let mut spans = Vec::new();
            for (i, (core_idx, current)) in chunk.iter().enumerate() {
                let color = self.color_scheme.get_gradient_color(*current);

                spans.push(Span::styled(
                    format!("C{:<2}", core_idx),
                    Style::default().fg(Color::White).bold()
                ));
                spans.push(Span::styled(
                    format!(" {:>3.0}% ", current),
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                ));

                let filled = ((current / 100.0) * bar_width as f64) as usize;
                let empty = bar_width.saturating_sub(filled);

                if filled > 0 {
                    spans.push(Span::styled(
                        "█".repeat(filled),
                        Style::default().fg(color)
                    ));
                }
                if empty > 0 {
                    spans.push(Span::styled(
                        "░".repeat(empty),
                        Style::default().fg(Color::DarkGray)
                    ));
                }

                if i < chunk.len() - 1 {
                    spans.push(Span::raw(" "));
                }
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}
