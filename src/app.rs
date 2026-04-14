use std::{io, time::Duration};

use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crate::collection::TopCollection;
use crate::data::temp::TempData;
use crate::data::os::OsInfo;
use crate::data::cpu::CpuInfo;
use crate::{event::handle_events};
use crate::draw::widgets::refresh_ticker::TickButton;
use crate::draw::widgets::process_table::ProcInfoPopup;
use crate::draw::widgets::process_taskbar::ProcessTaskBar;
use crate::draw::widgets::about_popup::AboutPopUp;
use crate::draw::widgets::disk_table::DiskTable;
use crate::draw::widgets::temp_widget::TempWidget;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize, Modifier},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub struct App {
    collection: TopCollection,
    temp_widget: TempWidget,
    disk_graph: DiskTable,
    duration_control: TickButton,
    process_taskbar: ProcessTaskBar,
    proc_info_popup: ProcInfoPopup,
    popup: AboutPopUp,
}

impl App {
    pub fn new() -> Self {
        let collection = TopCollection::new();
        let disk_graph = DiskTable::new();
        let duration_control = TickButton::new(Duration::from_millis(2000));
        let process_taskbar = ProcessTaskBar::new();

        let mut temp_widget = TempWidget::default();
        temp_widget.filter();

        let popup = AboutPopUp::default();

        Self {
            collection,
            temp_widget,
            disk_graph,
            duration_control,
            process_taskbar,
            proc_info_popup: ProcInfoPopup::new(),
            popup,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;

        loop {
            self.collection.refresh_timer();

            self.collection.update_time().await;
            self.collection.update_data();

            self.draw(terminal)?;

            if handle_events(
                &mut self.duration_control,
                &mut self.collection.process_tree,
                &mut self.process_taskbar,
                &mut self.proc_info_popup,
                &mut self.popup,
                &mut self.collection.sys,
            )? {
                break;
            }
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.proc_info_popup.refresh(&mut self.collection.sys);
        terminal.draw(|frame| {
            let instructions = Line::from(vec![
                " Quit ".red().bold().into(),
                "<Q/Esc> ".red().bold(),
            ]);

            let hostname_output = OsInfo::display_host_name().unwrap();

            let outer_block = Block::bordered()
                .title(Line::from(self.collection.ntp_time.clone()).centered())
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

            let num_cores = CpuInfo::num_cores(&mut self.collection.sys);
            let left_width = layout[0].width.saturating_sub(2) as usize;
            let label_width = 10;
            let min_bar_width = 10;
            let cores_per_row = (left_width / (label_width + min_bar_width)).max(1);
            let num_rows = (num_cores + cores_per_row - 1) / cores_per_row;
            let cpu_cores_height = (num_rows + 2).max(5) as u16;
            let cpu_info_height = (self.collection.cpu_model_lines.len().max(self.collection.cpu_cache_lines.len()).max(2) + 2) as u16;

            let temp_widget_height = if TempData::all_temps().is_some() {
                self.temp_widget.get_height().max(self.collection.temp_bar.get_height() + 1)
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

            let cpu_cores = CpuInfo::display_cores(&mut self.collection.sys)
                .unwrap_or_else(|| vec![String::from("No CPU data available")]);

            for core in cpu_cores {
                cpu_lines.push(Line::from(core));
            }

            let cpu_model_content_width = self.collection.cpu_model_lines.iter()
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
                Paragraph::new(self.collection.cpu_model_lines.clone())
                    .block(Block::new()
                        .borders(Borders::ALL)
                        .title("CPU Model")
                        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                cpu_model_area
            );

            let cpu_cache_content_width = self.collection.cpu_cache_lines.iter()
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
                Paragraph::new(self.collection.cpu_cache_lines.clone())
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
                Paragraph::new(self.collection.mem_lines.clone())
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
                self.collection.temp_bar.render(frame, temp_layout[1]);
            }

            self.collection.core_graph.render(frame, left_layout[1], &mut self.collection.sys);
            self.collection.total_cpu_bar.render(frame, left_layout[2]);

            self.collection.network_histogram.render(frame, left_layout[4]);

            let disk_height = self.disk_graph.get_height(&mut self.collection.disk_data);
            let right_layout = Layout::vertical([
                Constraint::Length(disk_height),
                Constraint::Min(10),
            ]).split(layout[1]);

            self.disk_graph.render(frame, right_layout[0], &mut self.collection.disk_data);

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

            self.collection.process_tree.render(frame, proc_split[0]);
            self.process_taskbar.render(frame, proc_split[1], self.collection.process_tree.selected_pid);

            self.proc_info_popup.render(frame, frame.area());
            self.popup.render(frame, frame.area());
        })?;
        Ok(())
    }
}
