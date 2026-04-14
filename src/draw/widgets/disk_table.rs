use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Row, Table, Cell},
    style::{Color, Style, Modifier},
    text::Line,
    layout::Constraint,
};

use crate::data::disk::DiskData;
use crate::data::os::OsInfo;
use crate::tools::units::format_bytes;

pub struct DiskTable;

impl DiskTable {
    pub fn new() -> Self {
        Self
    }

    pub fn get_height(&self, disk_data: &DiskData) -> u16 {
        (disk_data.disk_name.len() as u16 + 1 + 2).max(4)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, disk_data: &DiskData) {
        let kernel_output = format!("Kernel {}", OsInfo::display_kernel().unwrap());

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

        let rows: Vec<Row> = (0..disk_data.disk_name.len()).map(|i| {
            Row::new(vec![
                Cell::from(disk_data.disk_name[i].as_str()).style(Style::default().fg(Color::White)),
                Cell::from(disk_data.filesytem[i].as_str()).style(Style::default().fg(Color::Gray)),
                Cell::from(disk_data.mount[i].as_str()).style(Style::default().fg(Color::Gray)),
                Cell::from(format_bytes(disk_data.total[i])).style(Style::default().fg(Color::White)),
                Cell::from(format_bytes(disk_data.available[i])).style(Style::default().fg(Color::White)),
                Cell::from(format_bytes(disk_data.curr_read[i])).style(Style::default().fg(Color::Green)),
                Cell::from(format_bytes(disk_data.curr_write[i])).style(Style::default().fg(Color::Red)),
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
