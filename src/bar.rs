
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

pub enum BarColorScheme {
    Green,
    Yellow,
    Red,
}

impl BarColorScheme {
    fn get_color(&self, percentage: f64) -> Color {
        match self {
            BarColorScheme::Green => {
                if percentage < 50.0 {
                    Color::Green
                } else if percentage < 75.0 {
                    Color::Yellow
                } else {
                    Color::Red
                }
            }
            BarColorScheme::Yellow => Color::Yellow,
            BarColorScheme::Red => Color::Red,
        }
    }
}

pub struct TotalCoreBar {
    total_usage: f64,
    color_scheme: BarColorScheme,
}

impl TotalCoreBar {
    pub fn new(color_scheme: BarColorScheme) -> Self {
        Self {
            total_usage: 0.0,
            color_scheme,
        }
    }

    pub fn update(&mut self, core_usages: &[f64]) {
        if core_usages.is_empty() {
            self.total_usage = 0.0;
        } else {
            self.total_usage = core_usages.iter().sum::<f64>() / core_usages.len() as f64;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Total CPU Usage")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.width < 10 {
            return;
        }

        let color = self.color_scheme.get_color(self.total_usage);
        let bar_width = inner_area.width.saturating_sub(10) as usize;
        let filled = ((self.total_usage / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);

        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("CPU: {:>5.1}% ", self.total_usage),
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        ));

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

        let line = Line::from(spans);
        let paragraph = Paragraph::new(vec![line]);
        frame.render_widget(paragraph, inner_area);
    }
}