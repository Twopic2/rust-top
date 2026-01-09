use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize, Modifier},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::info::SystemInfo;

pub struct App {
    should_quit: bool,
    sys_info: SystemInfo,
    cpu_model_lines: Vec<Line<'static>>,
}

impl App {    
    const TICK_RATE: Duration = Duration::from_millis(250);

    pub fn new() -> Self {
        let sys_info = SystemInfo::new();

        let cpu_model_lines = if let Some(cpu_model) = sys_info.display_cpu_model() {
            cpu_model.into_iter()
                .map(|(key, value)| Line::from(format!("{}: {}", key, value)))
                .collect()
        } else {
            Vec::new()
        };

        Self {
            should_quit: false,
            sys_info,
            cpu_model_lines,
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
    
    fn draw(&mut self, frame: &mut Frame) {
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
        ]).split(inner_area);

        let cpu_layout = Layout::vertical([
            Constraint::Percentage(15),
            Constraint::Percentage(85),
        ]).split(layout[0]);

        let mut cpu_lines: Vec<Line> = Vec::new();

        let cpu_cores = self.sys_info.display_cores()
            .unwrap_or_else(|| vec![String::from("No CPU data available")]);

        for core in cpu_cores {
            cpu_lines.push(Line::from(core));
        }

        let memory_info = self.sys_info.display_memory();
        let memory_lines: Vec<Line> = memory_info.iter().map(|s| Line::from(s.as_str())).collect();

        frame.render_widget(
            Paragraph::new(self.cpu_model_lines.clone())
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("Cpu Model")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            cpu_layout[0]
        );

        frame.render_widget(
            Paragraph::new(cpu_lines)
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("CPU Info")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            cpu_layout[1]
        );
        frame.render_widget(
            Paragraph::new(memory_lines)
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("Memory Info")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            layout[1]
        );
    }
}
