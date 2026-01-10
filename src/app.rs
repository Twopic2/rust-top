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
use crate::graph::{MultiCoreGraph, ColorScheme};
use crate::bar::{TotalCoreBar, BarColorScheme};

pub struct App {
    should_quit: bool,
    sys_info: SystemInfo,
    cpu_model_lines: Vec<Line<'static>>,
    core_graph: MultiCoreGraph,
    total_cpu_bar: TotalCoreBar,
}

impl App {    
    const TICK_RATE: Duration = Duration::from_millis(2000);

    pub fn new() -> Self {
        let sys_info = SystemInfo::new();

        let cpu_model_lines = if let Some(cpu_model) = sys_info.display_cpu_model() {
            cpu_model.into_iter()
                .map(|(key, value)| Line::from(format!("{}: {}", key, value)))
                .collect()
        } else {
            Vec::new()
        };

        let num_cores = sys_info.num_cores();
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);

        Self {
            should_quit: false,
            sys_info,
            cpu_model_lines,
            core_graph,
            total_cpu_bar,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            self.sys_info.set_refresh_timer();

            let core_usages = self.sys_info.get_core_usages();

            for (i, usage) in core_usages.iter().enumerate() {
                self.core_graph.push(i, *usage);
            }

            self.total_cpu_bar.update(&core_usages);

            if let Some(mut freq_vec) = self.sys_info.display_cpu_frequency() {
                if let Some(freq) = freq_vec.pop() {
                    self.core_graph.set_cpu_frequency(freq);
                }
            }

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
            " Quit ".red().bold().into(),
            "<Q/Esc> ".red().bold(),
        ]);
        let outer_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, frame.area());

        let layout = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ]).split(inner_area);

        let num_cores = self.sys_info.num_cores();
        let left_width = layout[0].width.saturating_sub(2) as usize;
        let label_width = 10;
        let min_bar_width = 10;
        let cores_per_row = (left_width / (label_width + min_bar_width)).max(1);
        let num_rows = (num_cores + cores_per_row - 1) / cores_per_row;
        let cpu_cores_height = (num_rows + 2).max(5) as u16;

        let left_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(cpu_cores_height),
            Constraint::Length(3),
            Constraint::Min(0),
        ]).split(layout[0]);

        let mut cpu_lines: Vec<Line> = Vec::new();

        let cpu_cores = self.sys_info.display_cores()
            .unwrap_or_else(|| vec![String::from("No CPU data available")]);

        for core in cpu_cores {
            cpu_lines.push(Line::from(core));
        }

        let memory_info = self.sys_info.display_memory();
        let memory_lines: Vec<Line> = memory_info.iter().map(|s| Line::from(s.as_str())).collect();

        let cpu_model_content_width = self.cpu_model_lines.iter()
            .map(|line| line.to_string().len())
            .max()
            .unwrap_or(20) + 4; 

        let cpu_model_area = ratatui::layout::Rect {
            x: left_layout[0].x,
            y: left_layout[0].y,
            width: cpu_model_content_width.min(left_layout[0].width as usize) as u16,
            height: left_layout[0].height,
        };

        frame.render_widget(
            Paragraph::new(self.cpu_model_lines.clone())
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("CPU Model")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            cpu_model_area
        );

        self.core_graph.render(frame, left_layout[1]);

        self.total_cpu_bar.render(frame, left_layout[2]);

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
