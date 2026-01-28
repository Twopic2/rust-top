use std::{io, time::Duration};

use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crate::event::handle_events;
use crate::draw::misc::TickButton;
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
use crate::unix::disk::DiskData;
use crate::draw::graph::{MultiCoreGraph, DiskGraph, ColorScheme};
use crate::draw::bar::{TotalCoreBar, TempBar, BarColorScheme};
use crate::draw::histogram::NetworkHistogram;
use crate::draw::widget::TempWidget;

pub struct App {
    sys_info: SystemInfo,
    cpu_model_lines: Vec<Line<'static>>,
    cpu_cache_lines: Vec<Line<'static>>,
    mem_lines: Vec<Line<'static>>,
    core_graph: MultiCoreGraph,
    total_cpu_bar: TotalCoreBar,
    temp_widget: TempWidget,
    temp_bar: TempBar,
    network_histogram: NetworkHistogram,
    disk_data: DiskData,
    disk_graph: DiskGraph,
    duration_control: TickButton,
}

impl App {
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

        let memory_info = sys_info.display_memory();
        let mem_lines: Vec<Line<'static>> = memory_info.iter()
            .map(|s| Line::from(s.clone()))
            .collect();

        let num_cores = sys_info.num_cores();
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);
        let temp_widget = TempWidget::new();
        let temp_bar = TempBar::new(BarColorScheme::Green);
        let network_histogram = NetworkHistogram::new(60);
        let disk_data = DiskData::new();
        let disk_graph = DiskGraph::new();
        let duration_control = TickButton::new(Duration::from_millis(2000));

        Self {
            sys_info,
            cpu_model_lines,
            cpu_cache_lines,
            mem_lines,
            core_graph,
            total_cpu_bar,
            temp_widget,
            temp_bar,
            network_histogram,
            disk_data,
            disk_graph,
            duration_control,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;

        loop {
            self.sys_info.set_refresh_timer();

            let core_usages = self.sys_info.get_core_usages();

            for (i, usage) in core_usages.iter().enumerate() {
                self.core_graph.push(i, *usage);
            }

            self.total_cpu_bar.update(&core_usages);
            self.temp_bar.update();

            self.network_histogram.update();

            self.disk_data.refresh();
            self.disk_graph.update(&mut self.disk_data);

            let memory_info = self.sys_info.display_memory();
            self.mem_lines = memory_info.iter()
                .map(|s| Line::from(s.clone()))
                .collect();

            terminal.draw(|frame| self.draw(frame))?;

            if handle_events(&mut self.duration_control)? {
                break;
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
        
        let hostname_output = self.sys_info.display_host_name();

        let outer_block = Block::bordered()
            .title(title.centered())
            .title(Line::from(hostname_output).left_aligned())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, frame.area());

        let duration_ms = self.duration_control.get_duration().as_millis();
        let duration_text_len = format!("   - {}ms  +   ", duration_ms).len() as u16;
        let duration_area = ratatui::layout::Rect {
            x: frame.area().width.saturating_sub(duration_text_len + 2),
            y: 0,
            width: duration_text_len,
            height: 1,
        };
        self.duration_control.render(frame, duration_area);

        let layout = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(inner_area);

        let num_cores = self.sys_info.num_cores();
        let left_width = layout[0].width.saturating_sub(2) as usize;
        let label_width = 10;
        let min_bar_width = 10;
        let cores_per_row = (left_width / (label_width + min_bar_width)).max(1);
        let num_rows = (num_cores + cores_per_row - 1) / cores_per_row;
        let cpu_cores_height = (num_rows + 2).max(5) as u16;
        let cpu_info_height = (self.cpu_model_lines.len().max(self.cpu_cache_lines.len()).max(2) + 2) as u16;
        let temp_widget_height = self.temp_widget.get_height();

        let left_layout = Layout::vertical([
            Constraint::Length(cpu_info_height),
            Constraint::Length(cpu_cores_height),
            Constraint::Length(3),
            Constraint::Length(temp_widget_height),
            Constraint::Min(10),
        ]).split(layout[0]);

        let mut cpu_lines: Vec<Line> = Vec::new();

        let cpu_cores = self.sys_info.display_cores()
            .unwrap_or_else(|| vec![String::from("No CPU data available")]);

        for core in cpu_cores {
            cpu_lines.push(Line::from(core));
        }

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

        let cpu_cache_content_width = self.cpu_cache_lines.iter()
            .map(|line| line.to_string().len())
            .max()
            .unwrap_or(20) + 4;

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
            Paragraph::new(self.mem_lines.clone())
                .block(Block::new()
                    .borders(Borders::ALL)
                    .title("Memory")
                    .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
            mem_area
        );

        self.core_graph.render(frame, left_layout[1]);
        self.total_cpu_bar.render(frame, left_layout[2]);

        let temp_length = self.temp_widget.get_length();
        let temp_layout = Layout::horizontal([
            Constraint::Length(temp_length),
            Constraint::Min(0),
        ]).split(left_layout[3]);

        self.temp_widget.render(frame, temp_layout[0]);
        self.temp_bar.render(frame, temp_layout[1]);

        self.network_histogram.render(frame, left_layout[4]);
        

        let right_layout = Layout::vertical([
            Constraint::Min(10),
        ]).split(layout[1]);

        self.disk_graph.render(frame, right_layout[0]);
    }
}
