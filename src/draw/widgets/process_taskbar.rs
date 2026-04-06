use signal_hook::iterator::{SignalsInfo, exfiltrator::SignalOnly};
use sysinfo::{Pid, System};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::draw::widgets::process_table::{ProcessTable, ProcInfoPopup};

pub type Signals = SignalsInfo<SignalOnly>;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ProcessCommands {
    Select,
    Kill,
    Info,
}

pub struct ProcessTaskBar {
    pub command: ProcessCommands,
    last_render_area: Option<Rect>,
}

impl ProcessTaskBar {
    const BUTTONS: [(&'static str, ProcessCommands); 3] = [
        ("Select", ProcessCommands::Select),
        ("Kill",   ProcessCommands::Kill),
        ("Info", ProcessCommands::Info),
    ];

    pub fn new() -> Self {
        ProcessTaskBar {
            command: ProcessCommands::Select,
            last_render_area: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, selected_pid: u32) {
        self.last_render_area = Some(area);

        let mut spans: Vec<Span> = Vec::new();

        for (label, cmd) in Self::BUTTONS {
            let active = match(selected_pid != 0, cmd) {
                (true,  ProcessCommands::Kill | ProcessCommands::Info) => true,
                (false, ProcessCommands::Select) => true,
                _ => false,
            };

            let label_style = if active {
                Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            spans.push(Span::styled(format!("{} ", label), label_style));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    pub fn handle_click(&mut self, mouse_event: MouseEvent) -> Option<ProcessCommands> {
        let Some(area) = self.last_render_area else { return None };
        if mouse_event.kind != MouseEventKind::Down(MouseButton::Left) { return None; }

        let col = mouse_event.column;
        let row = mouse_event.row;
        if row < area.y || row >= area.y + area.height { return None; }
        if col < area.x || col >= area.x + area.width { return None; }

        let mut x = area.x;
        for (label, cmd) in Self::BUTTONS {
            let btn_width = (label.len() + 1) as u16;
            if col >= x && col < x + btn_width {
                self.command = cmd;
                return Some(cmd);
            }
            x += btn_width;
        }
        None
    }

    pub fn signal_process(&self, process_widget: &ProcessTable, info: &mut ProcInfoPopup, sys: &mut System) {
        let pid = process_widget.selected_pid;
        if pid == 0 { return; }
        match self.command {
            ProcessCommands::Select => {}
            ProcessCommands::Kill => Self::kill_proc(Pid::from_u32(pid), sys),
            ProcessCommands::Info => Self::info_proc(pid, info),
        }
    }

    fn kill_proc(pid: Pid, sys: &mut System) {
        if let Some(process) = sys.process(pid) {
            process.kill();
        }
    }

    fn info_proc(pid: u32, info: &mut ProcInfoPopup) {
        info.selected_pid = pid;
        info.visable = !info.visable;
    }
}
