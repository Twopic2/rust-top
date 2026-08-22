//! Shared helpers for the integration test suite.
//!
//! Each file in `tests/` is compiled as its own binary, so not every helper is
//! used by every file — hence the blanket `dead_code` allow.
#![allow(dead_code)]

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::{Frame, Terminal};
use rust_top::processes::processdata::{CollectProcessData, Io};
use sysinfo::System;

/// Draw one frame into an off-screen terminal and hand back the resulting buffer.
pub fn render<F>(width: u16, height: u16, draw: F) -> Buffer
where
    F: FnOnce(&mut Frame),
{
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(draw).expect("draw frame");
    terminal.backend().buffer().clone()
}

/// Flatten a buffer into newline-separated rows so tests can assert on text.
pub fn buffer_text(buf: &Buffer) -> String {
    let area = *buf.area();
    let mut out = String::new();
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            out.push_str(buf.cell((x, y)).map(|c| c.symbol()).unwrap_or(" "));
        }
        out.push('\n');
    }
    out
}

/// True when every cell in the buffer is blank.
pub fn buffer_is_blank(buf: &Buffer) -> bool {
    buffer_text(buf).chars().all(|c| c == ' ' || c == '\n')
}

/// A fully populated `System`. Expensive, so share it within a test where possible.
pub fn loaded_system() -> System {
    System::new_all()
}

/// Build a process row without touching the real system.
pub fn proc_row(pid: u32, program: &str, command: &str, cpu: f32, mem: f32) -> CollectProcessData {
    CollectProcessData {
        pid,
        command: command.to_string(),
        program: program.to_string(),
        mem_usage_percent: mem,
        cpu_usage_percent: cpu,
        disk_io: None,
        uid: Some(1000),
        user: "Base User",
    }
}

/// Same as [`proc_row`] but with disk IO attached.
pub fn proc_row_with_io(
    pid: u32,
    program: &str,
    command: &str,
    cpu: f32,
    mem: f32,
    read_bytes: u64,
    write_bytes: u64,
) -> CollectProcessData {
    let mut p = proc_row(pid, program, command, cpu, mem);
    p.disk_io = Some(Io {
        read_bytes,
        write_bytes,
    });
    p
}

/// The PIDs of a process list, in order.
pub fn pids(rows: &[CollectProcessData]) -> Vec<u32> {
    rows.iter().map(|p| p.pid).collect()
}
