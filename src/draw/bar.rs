use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

use crate::data::temp::TempData;

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

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
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

pub struct TempBar {
    temp_data: TempData,
    color_scheme: BarColorScheme,
    cpu_temp: Option<f32>,
    disk_temp: Option<f32>,
}

impl TempBar {
    pub fn new(color_scheme: BarColorScheme) -> Self {
        Self {
            temp_data: TempData::new(),
            color_scheme,
            cpu_temp: None,
            disk_temp: None,
        }
    }

    pub fn update(&mut self) {
        self.cpu_temp = None;
        self.disk_temp = None;

        if let Some(all_temps) = self.temp_data.get_all_temps() {
            let mut has_cpu = false;
            let mut has_disk = false;

            for (label, temp_opt) in all_temps {
                if let Some(temp) = temp_opt {
                    /* For macOS */
                    if !has_cpu && label.contains("tdie") {
                        self.cpu_temp = Some(temp);
                        has_cpu = true;
                    } else if !has_disk && label.contains("NAND") {
                        self.disk_temp = Some(temp);
                        has_disk = true;
                    }
                    /* For Unix*/
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Hardware Temperature")
            .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.width < 20 || inner_area.height < 2 {
            return;
        }

        let mut spans = Vec::new();

        if let Some(cpu_temp) = self.cpu_temp {
            let temp_percentage = (cpu_temp / 100.0 * 100.0).min(100.0).max(0.0);
            let color = self.color_scheme.get_color(temp_percentage as f64);

            spans.push(Span::styled(
                format!("CPU: {:>4.1}°C ", cpu_temp),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ));

            let bar_width = (inner_area.width as usize / 2).saturating_sub(14);
            let filled = ((temp_percentage / 100.0) * bar_width as f32) as usize;
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

            spans.push(Span::raw(" "));
        }

        if let Some(disk_temp) = self.disk_temp {
            let temp_percentage = (disk_temp / 100.0 * 100.0).min(100.0).max(0.0);
            let color = self.color_scheme.get_color(temp_percentage as f64);

            spans.push(Span::styled(
                format!("Disk: {:>4.1}°C ", disk_temp),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ));

            let bar_width = (inner_area.width as usize / 2).saturating_sub(15);
            let filled = ((temp_percentage / 100.0) * bar_width as f32) as usize;
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
        }

        if !spans.is_empty() {
            let line = Line::from(spans);
            let paragraph = Paragraph::new(vec![line]);
            frame.render_widget(paragraph, inner_area);
        }
    }
}
