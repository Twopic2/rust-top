use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};
use ratatui::Frame;

#[derive(Default)]
pub struct AboutPopUp {
    pub visable: bool,
}

impl AboutPopUp {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visable { return; }

        let blur_line = "░".repeat(area.width as usize);
        let blur_lines: Vec<Line> = (0..area.height)
            .map(|_| Line::from(blur_line.clone()))
            .collect();
        frame.render_widget(
            Paragraph::new(blur_lines)
                .style(Style::default().fg(Color::DarkGray).bg(Color::Black)),
            area,
        );

        let about_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(20));

        let popup_block = Block::bordered().title("About Rust-top").title_alignment(Alignment::Center);
        let inner = popup_block.inner(about_area);

        frame.render_widget(Clear, about_area);
        frame.render_widget(popup_block, about_area);

        let lines = vec![
            Line::from("Terminal system monitoring created with Ratui"),
            Line::from("If you have any suggestions contact Twopic2 on github"),
            Line::from(""),
            Line::from("Basic Keybinds"),
            Line::from("  Q / Esc  — Quit"),
            Line::from("  K        — Kill selected process"),
            Line::from("  /        — Search/filter processes"),
            Line::from("  + / =    — Increase refresh rate"),
            Line::from("  - / _    — Decrease refresh rate"),
        ];

        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
    }
}
