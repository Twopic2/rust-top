use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::{Block, Paragraph, Widget}};

use crate::data::voltage::Voltage;

pub struct VoltageUsage {
    pub voltages: Voltage,
}

impl VoltageUsage {
    pub fn new(voltages: Voltage) -> Self {    
        Self {
            voltages
        }
    }
}

impl Widget for VoltageUsage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = Vec::new();

        for volt in self.voltages.voltage_read_points {
            let voltage_point = Line::from(format!("{}: {}V", volt.0, volt.1));
            lines.push(voltage_point);
        }

        Paragraph::new(lines).block(Block::bordered().title("Voltage EndPoints")).render(area, buf);
    }
}
