use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::data::voltage::Voltage;

pub struct VoltageUsage {
    pub voltages: Voltage,
}

impl VoltageUsage {
    pub fn new(voltages: Voltage) -> Self {
        Self { voltages }
    }

    pub fn get_height(&self) -> u16 {
        (self.voltages.voltage_read_points.len().max(1) + 2).min(u16::MAX as usize) as u16
    }
}

impl Widget for &VoltageUsage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines: Vec<Line> = if self.voltages.voltage_read_points.is_empty() {
            vec![Line::styled("N/A", Style::default().fg(Color::DarkGray))]
        } else {
            self.voltages
                .voltage_read_points
                .iter()
                .map(|(label, value)| {
                    Line::from(vec![
                        Span::styled(
                            format!("{label}: "),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!("{value:.3} V"), Style::default().fg(Color::Green)),
                    ])
                })
                .collect()
        };

        Paragraph::new(lines)
            .block(
                Block::bordered().title("Voltage").title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .render(area, buf);
    }
}
