use ratatui::{
    buffer::Buffer,
    layout::Constraint,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

use crate::data::disk::DiskData;
use crate::data::os::OsInfo;
use crate::tools::units::format_bytes;

pub struct DiskTable<'a> {
    disk_data: &'a DiskData,
    kernel_output: String,
}

impl<'a> DiskTable<'a> {
    pub fn new(disk_data: &'a DiskData) -> Self {
        Self {
            disk_data,
            kernel_output: format!("Kernel {}", OsInfo::display_kernel().unwrap_or_default()),
        }
    }

    pub fn get_height(&self) -> u16 {
        (self.disk_data.disk_name.len() as u16 + 1 + 2).max(4)
    }
}

impl Widget for &DiskTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let disk_data = self.disk_data;
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Disk Usage")
            .title(Line::from(self.kernel_output.as_str()).right_aligned())
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.height < 2 || inner_area.width < 2 {
            return;
        }

        let header_cells = ["Disk", "FS", "Mount", "Total", "Available", "IO R", "IO W"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = (0..disk_data.disk_name.len())
            .map(|i| {
                Row::new(vec![
                    Cell::from(disk_data.disk_name[i].as_str())
                        .style(Style::default().fg(Color::White)),
                    Cell::from(disk_data.filesytem[i].as_str())
                        .style(Style::default().fg(Color::Gray)),
                    Cell::from(disk_data.mount[i].as_str()).style(Style::default().fg(Color::Gray)),
                    Cell::from(format_bytes(disk_data.total[i]))
                        .style(Style::default().fg(Color::White)),
                    Cell::from(format_bytes(disk_data.available[i]))
                        .style(Style::default().fg(Color::White)),
                    Cell::from(format_bytes(disk_data.curr_read[i]))
                        .style(Style::default().fg(Color::Green)),
                    Cell::from(format_bytes(disk_data.curr_write[i]))
                        .style(Style::default().fg(Color::Red)),
                ])
            })
            .collect();

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

        Widget::render(table, inner_area, buf);
    }
}
