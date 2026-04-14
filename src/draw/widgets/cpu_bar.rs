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

// Temp bar lists out the proper temps in f32.
pub struct TempBar {
    color_scheme: BarColorScheme,
    cpu_temp: Option<f32>,
    disk_temp: Option<f32>,
    ddr_temp: Option<f32>,
    nic_temp: Option<f32>,
}

impl TempBar {
    pub fn new(color_scheme: BarColorScheme) -> Self {
        Self {
            color_scheme,
            cpu_temp: None,
            disk_temp: None,
            ddr_temp: None,
            nic_temp: None,
        }
    }

    pub fn update(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(all_temps) = TempData::all_temps() {
            let mut has_cpu = false;
            let mut has_disk = false;

            for (label, temp_opt) in all_temps {
                if let Some(temp) = temp_opt {
                    if !has_cpu && label.contains("tdie") {
                        self.cpu_temp = Some(temp);
                        has_cpu = true;
                    } else if !has_disk && label.contains("NAND") {
                        self.disk_temp = Some(temp);
                        has_disk = true;
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        if let Some(all_temps) = TempData::all_temps() {
            for (label, temp_out) in all_temps {
                if let Some(temp) = temp_out {
                    if label.contains("coretemp") || label.contains("Package id") || label.contains("k10temp") || label.contains("Tdie") {
                        self.cpu_temp = Some(temp);
                    } else if label.contains("nvme") {
                        self.disk_temp = Some(temp);
                    } else if label.contains("iwlwifi") && label.contains("MT") // MediaTek 
                    {
                        self.nic_temp = Some(temp);
                    } else if label.contains("spd") {
                        self.ddr_temp = Some(temp);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")] 
    pub fn get_height(&self) -> u16 {
        let count = [self.cpu_temp, self.disk_temp, self.ddr_temp, self.ddr_temp]
            .iter()
            .filter(|t| t.is_some())
            .count();

        let rows = (count + 1) / 2; 
        (rows as u16).max(1) + 2 
    }

    #[cfg(target_os = "macos")]
    pub fn get_height(&self) -> u16 {
        let count = match (self.cpu_temp.is_some(), self.disk_temp.is_some()) {
            (true, true) => 2 as u16,
            (true, false) => 1 as u16,
            (false, true) => 1 as u16,
            (false, false) => 0 as u16,
        };
        
        let rows = (count + 1) / 2; 
        (rows as u16).max(1) + 2  
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

        let half_width = inner_area.width as usize / 2;

        let sensors: &[(&str, Option<f32>)] = &[
            ("CPU", self.cpu_temp),
            ("Disk", self.disk_temp),
            ("Nic", self.nic_temp),
            ("DDR", self.ddr_temp),
        ];

        let active: Vec<_> = sensors.iter()
            .filter_map(|(label, temp_opt)| temp_opt.map(|t| (*label, t)))
            .collect();

        let mut lines: Vec<Line> = Vec::new();
        for chunk in active.chunks(2) {
            let mut spans = Vec::new();
            for (label, temp) in chunk {
                let bar_width = half_width.saturating_sub(label.len() + 11);
                spans.extend(Self::temp_bar_spans(label, *temp, bar_width, &self.color_scheme));
                spans.push(Span::raw(" "));
            }
            lines.push(Line::from(spans));
        }

        if !lines.is_empty() {
            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, inner_area);
        }
    }

    fn temp_bar_spans(label: &str, temp: f32, bar_width: usize, color_scheme: &BarColorScheme) -> Vec<Span<'static>> {
        let temp_percentage = (temp / 100.0 * 100.0).min(100.0).max(0.0);
        let color = color_scheme.get_color(temp_percentage as f64);
        let filled = ((temp_percentage / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);

        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{}: {:>4.1}°C ", label, temp),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        if filled > 0 {
            spans.push(Span::styled("█".repeat(filled), Style::default().fg(color)));
        }
        if empty > 0 {
            spans.push(Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)));
        }
        spans
    }
}
