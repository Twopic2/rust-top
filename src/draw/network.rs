use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};

pub struct NetworkChart {
    curr_rx: u64,
    curr_tx: u64,
    total_rx: u64,
    total_tx: u64,
}

impl NetworkChart {
    pub fn new() -> Self {
        Self {
            curr_rx: 0,
            curr_tx: 0,
            total_rx: 0,
            total_tx: 0,
        }
    }

    pub fn update(&mut self, curr_rx: u64, curr_tx: u64, total_rx: u64, total_tx: u64) {
        self.curr_rx = curr_rx;
        self.curr_tx = curr_tx;
        self.total_rx = total_rx;
        self.total_tx = total_tx;
    }

    fn format_rate(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.1}Gb/s", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.1}Mb/s", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.1}Kb/s", bytes as f64 / 1024.0)
        } else {
            format!("{}b/s", bytes)
        }
    }

    fn format_total(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.1}MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else {
            format!("{}B", bytes)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Network")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let rx_line = Line::from(vec![
            Span::styled("RX: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:<12}", Self::format_rate(self.curr_rx)),
                Style::default().fg(Color::Yellow)
            ),
            Span::styled("All: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                Self::format_total(self.total_rx),
                Style::default().fg(Color::Yellow)
            ),
        ]);

        let tx_line = Line::from(vec![
            Span::styled("TX: ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{:<12}", Self::format_rate(self.curr_tx)),
                Style::default().fg(Color::Green)
            ),
            Span::styled("All: ", Style::default().fg(Color::Green)),
            Span::styled(
                Self::format_total(self.total_tx),
                Style::default().fg(Color::Green)
            ),
        ]);

        let paragraph = Paragraph::new(vec![rx_line, tx_line]);
        frame.render_widget(paragraph, inner_area);
    }
}
