use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    text::Span,
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize, Modifier},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub struct App {
    should_quit: bool,
}

impl App {    
    const TICK_RATE: Duration = Duration::from_millis(250);

    pub fn new() -> Self {
        Self {
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_keystrokes()?;

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn handle_keystrokes(&mut self) -> io::Result<()> {
        if event::poll(Self::TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
    
    fn draw(&self, frame: &mut Frame) {
        let title = Line::from("Rust-Top: Terminal Top in Rust".bold());
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q/Esc> ".blue().bold(),
        ]);
        let outer_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, frame.area());

        let layout = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(inner_area);

        let cpu_style = Span::styled("Cpu", Style::default().fg(Color::Red)).add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC);
        let memory_style = Span::styled("Memory", Style::default().fg(Color::Red)).add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC);

        frame.render_widget(
            Paragraph::new(cpu_style)
                .block(Block::new().borders(Borders::ALL)),
            layout[0]
        );
        frame.render_widget(
            Paragraph::new(memory_style)
                .block(Block::new().borders(Borders::ALL)),
            layout[1]
        );
    }
}
