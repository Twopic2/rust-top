use std::{io, time::Duration};

use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use sysinfo::System;
use crate::data::temp::TempData;
use crate::{event::handle_events};
use crate::draw::widgets::refresh_ticker::TickButton;
use crate::draw::widgets::process_table::{ProcessTable, ProcInfoPopup};
use crate::draw::widgets::process_taskbar::ProcessTaskBar;
use crate::draw::widgets::about_popup::AboutPopUp;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize, Modifier},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

#[cfg(target_os = "macos")]
use crate::data::darwin::cache::CacheMac;

use crate::data::cpu::CpuInfo;
use crate::data::mem::MemInfo;
use crate::data::os::OsInfo;
use crate::data::clock::local_time;
use crate::data::disk::DiskData;
use crate::draw::widgets::cpu_graph::{MultiCoreGraph, ColorScheme};
use crate::draw::widgets::disk_table::DiskTable;
use crate::draw::widgets::cpu_bar::{TotalCoreBar, TempBar, BarColorScheme};
use crate::draw::widgets::network_graph::NetworkGraph;
use crate::draw::widgets::temp_widget::TempWidget;

pub struct App {
    cpu_model_lines: Vec<Line<'static>>,
    cpu_cache_lines: Vec<Line<'static>>,
    mem_lines: Vec<Line<'static>>,
    core_graph: MultiCoreGraph,
    total_cpu_bar: TotalCoreBar,
    temp_bar: TempBar,
    temp_widget: TempWidget,
    network_histogram: NetworkGraph,
    disk_data: DiskData,
    disk_graph: DiskTable,
    duration_control: TickButton,
    process_tree: ProcessTable,
    process_taskbar: ProcessTaskBar,
    proc_info_popup: ProcInfoPopup,
    sys: System,
    popup: AboutPopUp,
    ntp_time: String,
}

impl App {
    pub fn new() -> Self {
        let mut sys = System::new_all();

        let cpu_model_lines = if let Some(cpu_model) = CpuInfo::display_cpu_model(&mut sys) {
            cpu_model.into_iter()
                .map(|(key, value)| Line::from(format!("{}: {}", key, value)))
                .collect()
        } else {
            Vec::new()
        };

        #[cfg(target_os = "macos")]
        let cpu_cache_lines: Vec<Line<'static>> = CacheMac::cache_lines();

        #[cfg(not(target_os = "macos"))]
        let cpu_cache_lines = if let Some(cpu_cache) = CpuInfo::display_cpu_cache() {
            let cache_str = cpu_cache.into_iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<_>>()
                .join(" | ");
            vec![Line::from(cache_str)]
        } else {
            vec![Line::from("Cache info not available")]
        };

        let mem_lines: Vec<Line<'static>> = if let Some(mem_info) = MemInfo::display_memory(&mut sys) {
            mem_info.into_iter().map(| str | Line::from(format!("{}", str))).collect::<Vec<_>>()
        } else {
            vec![Line::from("No mem info")]
        };

        let num_cores = CpuInfo::num_cores(&mut sys);
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);
        let temp_bar = TempBar::new(BarColorScheme::Green);
        let network_histogram = NetworkGraph::new(60);
        let disk_data = DiskData::default();
        let disk_graph = DiskTable::new();
        let duration_control = TickButton::new(Duration::from_millis(2000));
        let process_tree = ProcessTable::new();
        let process_taskbar = ProcessTaskBar::new();

        let mut temp_widget = TempWidget::default();
        temp_widget.filter();

        let popup = AboutPopUp::default();

        Self {
            cpu_model_lines,
            cpu_cache_lines,
            mem_lines,
            core_graph,
            total_cpu_bar,
            temp_bar,
            temp_widget,
            network_histogram,
            disk_data,
            disk_graph,
            duration_control,
            process_tree,
            process_taskbar,
            proc_info_popup: ProcInfoPopup::new(),
            sys,
            popup,
            ntp_time: "--:--:--".to_string(),
        }
    }

    pub fn update_data(&mut self) {
        let core_usages = CpuInfo::get_core_usages(&mut self.sys);

        for (i, usage) in core_usages.iter().enumerate() {
            self.core_graph.cores.insert(i, *usage);
        }

        self.total_cpu_bar.update(&core_usages);
        self.temp_bar.update();

        self.network_histogram.update();

        self.disk_data.refresh(&mut self.sys);
        self.disk_data.collect_all(&mut self.sys);

        self.process_tree.refresh(&mut self.sys);
    }

    async fn update_time(&mut self) {
        self.ntp_time = local_time().await;
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;

        loop {
            CpuInfo::set_refresh_timer(&mut self.sys);

            self.update_time().await;
            self.update_data();

            self.draw(terminal)?;

            if handle_events(&mut self.duration_control, &mut self.process_tree, &mut self.process_taskbar, &mut self.proc_info_popup, &mut self.popup, &mut self.sys)? {
                break;
            }
        }
        Ok(())
    }
    
    fn draw(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.proc_info_popup.refresh(&mut self.sys);
        terminal.draw(|frame| {
            let instructions = Line::from(vec![
                " Quit ".red().bold().into(),
                "<Q/Esc> ".red().bold(),
            ]);

            let hostname_output = OsInfo::display_host_name().unwrap();

            let outer_block = Block::bordered()
                .title(Line::from(self.ntp_time.clone()).centered())
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

            let num_cores = CpuInfo::num_cores(&mut self.sys);
            let left_width = layout[0].width.saturating_sub(2) as usize;
            let label_width = 10;
            let min_bar_width = 10;
            let cores_per_row = (left_width / (label_width + min_bar_width)).max(1);
            let num_rows = (num_cores + cores_per_row - 1) / cores_per_row;
            let cpu_cores_height = (num_rows + 2).max(5) as u16;
            let cpu_info_height = (self.cpu_model_lines.len().max(self.cpu_cache_lines.len()).max(2) + 2) as u16;

            let temp_widget_height = if TempData::all_temps().is_some() {
                self.temp_widget.get_height().max(self.temp_bar.get_height())
            } else {
                0
            };

            let left_layout = Layout::vertical([
                Constraint::Length(cpu_info_height),
                Constraint::Length(cpu_cores_height),
                Constraint::Length(5),
                Constraint::Length(temp_widget_height),
                Constraint::Min(10),
            ]).split(layout[0]);

            let mut cpu_lines: Vec<Line> = Vec::new();

            let cpu_cores = CpuInfo::display_cores(&mut self.sys)
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

            if TempData::all_temps().is_some() {
                let temp_length = self.temp_widget.get_length();
                let temp_layout = Layout::horizontal([
                    Constraint::Length(temp_length),
                    Constraint::Min(0),
                ]).split(left_layout[3]);
                self.temp_widget.render(frame, temp_layout[0]);
                self.temp_bar.render(frame, temp_layout[1]);
            }

            self.core_graph.render(frame, left_layout[1], &mut self.sys);
            self.total_cpu_bar.render(frame, left_layout[2]);

            self.network_histogram.render(frame, left_layout[4]);

            let disk_height = self.disk_graph.get_height(&mut self.disk_data);
            let right_layout = Layout::vertical([
                Constraint::Length(disk_height),
                Constraint::Min(10),
            ]).split(layout[1]);

            self.disk_graph.render(frame, right_layout[0], &mut self.disk_data);

            let proc_block = Block::new()
                .borders(Borders::ALL)
                .title("Processes")
                .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
            let proc_inner = proc_block.inner(right_layout[1]);
            frame.render_widget(proc_block, right_layout[1]);

            let proc_split = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(1),
            ]).split(proc_inner);



            self.process_tree.render(frame, proc_split[0]);
            self.process_taskbar.render(frame, proc_split[1], self.process_tree.selected_pid);

            self.proc_info_popup.render(frame, frame.area());
            self.popup.render(frame, frame.area());
        })?;
        Ok(())
    }
}
