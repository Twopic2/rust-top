use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

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
    cores: Vec<Vec<f64>>,
    color_scheme: ColorScheme,
    cpu_frequency: Option<String>,
}

impl MultiCoreGraph {
    pub fn new(num_cores: usize, color_scheme: ColorScheme) -> Self {
        Self {
            cores: vec![Vec::new(); num_cores],
            color_scheme,
            cpu_frequency: None,
        }
    }

    pub fn push(&mut self, core_index: usize, usage: f64) {
        if let Some(core_data) = self.cores.get_mut(core_index) {
            core_data.push(usage);
        }
    }

    fn trim_to_width(&mut self, max_points: usize) {
        for core_data in &mut self.cores {
            if core_data.len() > max_points {
                let excess = core_data.len() - max_points;
                core_data.drain(0..excess);
            }
        }
    }

    pub fn set_cpu_frequency(&mut self, frequency: String) {
        self.cpu_frequency = Some(frequency);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title2 = if let Some(ref freq) = self.cpu_frequency {
            format!("{} MHz", freq)
        } else {
            "0 MHz".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Cpu Cores")
            .title(Line::from(title2).right_aligned())
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 2 || inner_area.width < 2 {
            return;
        }

        let label_width = 10;
        let min_bar_width = 10;
        let cores_per_row = (inner_area.width as usize / (label_width + min_bar_width)).max(1);
        let bar_width = (inner_area.width as usize / cores_per_row).saturating_sub(label_width);

        let max_points = bar_width.max(10);

        self.trim_to_width(max_points);

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

        for chunk_idx in 0..(self.cores.len() + cores_per_row - 1) / cores_per_row {
            if lines.len() >= height {
                break;
            }

            let start_idx = chunk_idx * cores_per_row;
            let end_idx = (start_idx + cores_per_row).min(self.cores.len());
            let chunk = &self.cores[start_idx..end_idx];

            let mut spans = Vec::new();

            for (i, core_data) in chunk.iter().enumerate() {
                let core_idx = start_idx + i;
                let current = core_data.last().unwrap_or(&0.0);
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
