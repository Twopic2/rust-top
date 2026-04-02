use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Row, Table, Cell},
    style::{Color, Style, Modifier},
    text::Line,
    layout::Constraint,
};

use sysinfo::System;
use crate::data::{disk::DiskData, info::OsInfo};
use crate::data::info::SystemInfo;
use crate::tools::units::format_bytes;

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

pub struct DiskTable {
    entries: Vec<DiskDisplayEntry>,
    sys_info: SystemInfo,
}

impl DiskTable {
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
        for i in 0..disk_data.disk_name.len() {
            self.entries.push(DiskDisplayEntry {
                name: disk_data.disk_name[i].clone(),
                filesystem: disk_data.filesytem[i].clone(),
                mount: disk_data.mount[i].clone(),
                total: disk_data.total[i],
                available: disk_data.available[i],
                io_read: disk_data.curr_read[i],
                io_write: disk_data.curr_write[i],
            });
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

        let header_cells = ["Disk", "FS", "Mount", "Total", "Available", "IO R", "IO W"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = self.entries.iter().map(|entry| {
            Row::new(vec![
                Cell::from(entry.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(entry.filesystem.clone()).style(Style::default().fg(Color::Gray)),
                Cell::from(entry.mount.clone()).style(Style::default().fg(Color::Gray)),
                Cell::from(format_bytes(entry.total)).style(Style::default().fg(Color::White)),
                Cell::from(format_bytes(entry.available)).style(Style::default().fg(Color::White)),
                Cell::from(format_bytes(entry.io_read)).style(Style::default().fg(Color::Green)),
                Cell::from(format_bytes(entry.io_write)).style(Style::default().fg(Color::Red)),
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
            .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(table, inner_area);
    }
}
