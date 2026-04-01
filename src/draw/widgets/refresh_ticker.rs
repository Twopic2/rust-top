use ratatui::{
    layout::Rect,
    style::{Color, Style, Modifier},
    text::Span,
    widgets::Paragraph,
    Frame,
};
use std::time::Duration;
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

#[derive(Debug, PartialEq, Eq)]
pub enum TickCounter {
    Increment,
    Decrement,
    None,
}

#[derive(Debug, Clone)]
pub struct TickButton {
    duration: Duration,
    last_render_area: Option<Rect>,
}

impl TickButton {
    const MIN_TICK_RATE: Duration = Duration::from_millis(1000);
    const MAX_TICK_RATE: Duration = Duration::from_millis(5000);
    const TICK_RATE_STEP: Duration = Duration::from_millis(100);

    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            last_render_area: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.last_render_area = Some(area);

        let duration_ms = self.duration.as_millis();

        let text = vec![
            Span::styled(" - ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}ms ", duration_ms), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" + ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ];

        let paragraph = Paragraph::new(ratatui::text::Line::from(text));
        frame.render_widget(paragraph, area);
    }

    pub fn duration_click(&self, mouse_event: MouseEvent) -> TickCounter {
        if let Some(area) = self.last_render_area {
            if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                let col = mouse_event.column;
                let row = mouse_event.row;

                if row >= area.y && row < area.y + area.height
                   && col >= area.x && col < area.x + area.width {

                    let relative_col = col - area.x;

                    if relative_col < 3 {
                        return TickCounter::Decrement;
                    }

                    let duration_ms = self.duration.as_millis();
                    let text_len = format!(" - {}ms  + ", duration_ms).len() as u16;

                    if relative_col >= text_len - 3 {
                        return TickCounter::Increment;
                    }
                }
            }
        }
        TickCounter::None
    }

    pub fn increment(&mut self) {
        let new_rate = self.duration + Self::TICK_RATE_STEP;
        if new_rate <= Self::MAX_TICK_RATE {
            self.duration = new_rate;
        }
    }

    pub fn decrement(&mut self) {
        let new_rate = self.duration.saturating_sub(Self::TICK_RATE_STEP);
        if new_rate >= Self::MIN_TICK_RATE {
            self.duration = new_rate;
        }
    }

    pub fn get_duration(&self) -> Duration {
        self.duration
    }
}
