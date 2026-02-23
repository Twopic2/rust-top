use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Row, Table, Cell},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    layout::Constraint,
};

use sysinfo::System;
use crate::data::{disk::DiskData, info::OsInfo};
use crate::data::info::SystemInfo;

const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;

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
}

impl MultiCoreGraph {
    pub fn new(num_cores: usize, color_scheme: ColorScheme) -> Self {
        Self {
            cores: vec![Vec::new(); num_cores],
            color_scheme,
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

#[derive(Debug)]
pub struct DiskDisplayEntry {
    pub name: String,
    pub filesystem: String,
    pub mount: String,
    pub total: u64,
    pub available: u64,
    pub io_read: u64,
    pub io_write: u64,
}

pub struct DiskGraph {
    entries: Vec<DiskDisplayEntry>,
    sys_info: SystemInfo,
}

impl DiskGraph {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            sys_info: SystemInfo,
        }
    }

    pub fn get_height(&self) -> u16 {
        (self.entries.len() as u16 + 1 + 2).max(4)
    }

    pub fn update(&mut self, disk_data: &mut DiskData, sys: &mut System) {
        disk_data.collect_all(sys);

        self.entries.clear();
        for i in 0..disk_data.len() {
            self.entries.push(DiskDisplayEntry {
                name: disk_data.get_disks()[i].clone(),
                filesystem: disk_data.get_filesystems()[i].clone(),
                mount: disk_data.get_mounts()[i].clone(),
                total: disk_data.get_totals()[i],
                available: disk_data.get_available()[i],
                io_read: disk_data.get_reads()[i],
                io_write: disk_data.get_writes()[i],
            });
        }
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let kernel_output = self.sys_info.display_kernel();
        let kernel_output = format!("Kernel {}", kernel_output);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Disk Usage")
            .title(Line::from(kernel_output).right_aligned())
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 2 || inner_area.width < 2 {
            return;
        }

        let header_cells = ["Disk", "FS", "Mount", "Total", "Available","IO R", "IO W"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = self.entries.iter().map(|entry| {
            Row::new(vec![
                Cell::from(entry.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(entry.filesystem.clone()).style(Style::default().fg(Color::Gray)),
                Cell::from(entry.mount.clone()).style(Style::default().fg(Color::Gray)),
                Cell::from(Self::format_bytes(entry.total)).style(Style::default().fg(Color::White)),
                Cell::from(Self::format_bytes(entry.available)).style(Style::default().fg(Color::White)),
                Cell::from(Self::format_bytes(entry.io_read)).style(Style::default().fg(Color::Green)),
                Cell::from(Self::format_bytes(entry.io_write)).style(Style::default().fg(Color::Red)),
            ])
        }).collect();

        let widths = [
            Constraint::Percentage(12),
            Constraint::Percentage(10),
            Constraint::Percentage(18),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
            ;

        frame.render_widget(table, inner_area);
    }
}

