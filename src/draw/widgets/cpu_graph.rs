use indexmap::IndexMap;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
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
    pub cores: IndexMap<usize, f64>,
    pub color_scheme: ColorScheme,
    cpu_frequency_mhz: u64,
}

impl MultiCoreGraph {
    pub fn new(num_cores: usize, color_scheme: ColorScheme) -> Self {
        let cores = (0..num_cores).map(|i| (i, 0.0)).collect();
        Self {
            cores,
            color_scheme,
            cpu_frequency_mhz: 0,
        }
    }

    /// Store the CPU snapshot before rendering, with frequency in MHz.
    pub fn update(&mut self, core_usages: &[f64], cpu_frequency_mhz: u64) {
        self.cores.clear();
        self.cores.extend(core_usages.iter().copied().enumerate());
        self.cpu_frequency_mhz = cpu_frequency_mhz;
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
                    Style::default().fg(Color::White).bold(),
                ));
                spans.push(Span::styled(
                    format!(" {:>3.0}% ", current),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));

                let filled = ((current / 100.0) * bar_width as f64) as usize;
                let empty = bar_width.saturating_sub(filled);

                if filled > 0 {
                    spans.push(Span::styled("█".repeat(filled), Style::default().fg(color)));
                }
                if empty > 0 {
                    spans.push(Span::styled(
                        "░".repeat(empty),
                        Style::default().fg(Color::DarkGray),
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

impl Widget for &MultiCoreGraph {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let formatted_freq = format!("{:.2} GHz", self.cpu_frequency_mhz as f64 / 1000.0);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Cpu Cores")
            .title(Line::from(formatted_freq).right_aligned())
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.height < 2 || inner_area.width < 2 {
            return;
        }

        let lines = self.generate_core_grid(inner_area.width as usize, inner_area.height as usize);
        Paragraph::new(lines).render(inner_area, buf);
    }
}
