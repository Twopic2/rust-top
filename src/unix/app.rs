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

#[cfg(target_os = "macos")]
use crate::unix::darwin::cache::CacheMac;

use crate::unix::info::{OsInfo, SystemInfo};
use crate::draw::graph::{MultiCoreGraph, ColorScheme};
use crate::draw::bar::{TotalCoreBar, BarColorScheme};
use crate::draw::histogram::NetworkHistogram;

pub struct App {
    should_quit: bool,
    sys_info: SystemInfo,
    cpu_model_lines: Vec<Line<'static>>,
    cpu_cache_lines: Vec<Line<'static>>,
    mem_lines: Vec<Line<'static>>,
    core_graph: MultiCoreGraph,
    total_cpu_bar: TotalCoreBar,
    network_histogram: NetworkHistogram,
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

        #[cfg(target_os = "macos")]
        let cache_levels = CacheMac::cache_levels();          

        #[cfg(target_os = "macos")]
        let cpu_cache_lines: Vec<Line<'static>> = if !cache_levels.is_empty() {
            let p_cores: Vec<_> = cache_levels.iter().filter(|s| s.starts_with("P-")).cloned().collect();
            let e_cores: Vec<_> = cache_levels.iter().filter(|s| s.starts_with("E-")).cloned().collect();
            let mut lines = Vec::new();
            if !p_cores.is_empty() {
                lines.push(Line::from(p_cores.join(" | ")));
            }
            if !e_cores.is_empty() {
                lines.push(Line::from(e_cores.join(" | ")));
            }
            if lines.is_empty() {
                vec![Line::from(cache_levels.join(" | "))]
            } else {
                lines
            }
        } else {
            vec![Line::from("Apple Cache not here")]
        };

        #[cfg(not(target_os = "macos"))]
        let cpu_cache_lines = if let Some(cpu_cache) = sys_info.display_cpu_cache() {
            let cache_str = cpu_cache.into_iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<_>>()
                .join(" | ");
            vec![Line::from(cache_str)]
        } else {
            vec![Line::from("Cache info not available")]
        };

        let mem_lines: Vec<Line<'static>> = Vec::new();

        let num_cores = sys_info.num_cores();
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);
        let network_histogram = NetworkHistogram::new(60);

        Self {
            should_quit: false,
            sys_info,
            cpu_model_lines,
            cpu_cache_lines,
            mem_lines,
            core_graph,
            total_cpu_bar,
            network_histogram,
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

            self.network_histogram.update();

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

        let kernel_output = self.sys_info.display_kernel().join(" ");
        let hostname_output = self.sys_info.display_host_name().join(" ");

        let outer_block = Block::bordered()
            .title(title.centered())
            .title(Line::from(hostname_output).left_aligned())
            .title_bottom(instructions.centered())
            .title(Line::from(kernel_output).right_aligned())
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

        let cpu_info_height = (self.cpu_model_lines.len().max(self.cpu_cache_lines.len()).max(2) + 2) as u16;

        let left_layout = Layout::vertical([
            Constraint::Length(cpu_info_height),
            Constraint::Length(cpu_cores_height),
            Constraint::Length(3),
            Constraint::Min(10),
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

        #[cfg(target_os = "macos")]
        let cpu_cache_content_width = self.cpu_cache_lines.iter()
            .map(|line| line.to_string().len())
            .max()
            .unwrap_or(20) + 4;

        #[cfg(target_os = "macos")]
        let cpu_cache_area = ratatui::layout::Rect {
            x: cpu_model_area.x + cpu_model_area.width,
            y: left_layout[0].y,
            width: cpu_cache_content_width.min((left_layout[0].width - cpu_model_area.width) as usize) as u16,
            height: left_layout[0].height,
        }; 

        #[cfg(not(target_os = "macos"))]
        let cpu_cache_content_width = self.cpu_cache_lines.iter()
            .map(|line| line.to_string().len())
            .max()
            .unwrap_or(20) + 4;

        #[cfg(not(target_os = "macos"))]
        let cpu_cache_area = ratatui::layout::Rect {
            x: cpu_model_area.x + cpu_model_area.width,
            y: left_layout[0].y,
            width: cpu_cache_content_width.min((left_layout[0].width - cpu_model_area.width) as usize) as u16,
            height: left_layout[0].height,
        };

        frame.render_widget(
            Paragraph::new(self.cpu_cache_lines.clone())
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("CPU Cache")
                    .title_style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
            cpu_cache_area
        );

        let remaining_width = left_layout[0].width.saturating_sub(cpu_model_area.width + cpu_cache_area.width);

        let mem_area = ratatui::layout::Rect {
            x: cpu_cache_area.x + cpu_cache_area.width,
            y: left_layout[0].y,
            width: remaining_width,
            height: left_layout[0].height,
        };

        frame.render_widget(
            Paragraph::new(memory_lines.clone())
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("Memory")
                    .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
            mem_area
        );

        self.core_graph.render(frame, left_layout[1]);
        self.total_cpu_bar.render(frame, left_layout[2]);
        self.network_histogram.render(frame, left_layout[3]);

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
