//! `draw::widgets::process_taskbar` — button hit boxes and command dispatch.
//!
//! Note: the Kill path is deliberately never exercised with a live pid; these
//! tests only cover the Select/Info branches and the pid == 0 guard.

mod common;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use rust_top::draw::widgets::process_table::{ProcInfoPopup, ProcessTable};
use rust_top::draw::widgets::process_taskbar::{ProcessCommands, ProcessTaskBar};
use sysinfo::System;

fn left_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

/// Render once so `handle_click` has a hit box. Buttons are laid out as
/// "Select " (7 cols), "Kill " (5), "Info " (5).
fn rendered(area: Rect, selected_pid: u32) -> ProcessTaskBar {
    let mut bar = ProcessTaskBar::new();
    let _ = common::render(40, 3, |f| bar.render(f, area, selected_pid));
    bar
}

// --------------------------------------------------------------- construction

#[test]
fn new_taskbar_defaults_to_select() {
    assert_eq!(ProcessTaskBar::new().command, ProcessCommands::Select);
}

// ------------------------------------------------------------------ hit boxes

#[test]
fn clicking_before_the_first_render_does_nothing() {
    let mut bar = ProcessTaskBar::new();
    assert_eq!(bar.handle_click(left_click(1, 0)), None);
}

#[test]
fn each_button_owns_its_column_range() {
    let mut bar = rendered(Rect::new(0, 0, 20, 1), 0);

    for col in 0..7 {
        assert_eq!(bar.handle_click(left_click(col, 0)), Some(ProcessCommands::Select), "col {col}");
    }
    for col in 7..12 {
        assert_eq!(bar.handle_click(left_click(col, 0)), Some(ProcessCommands::Kill), "col {col}");
    }
    for col in 12..17 {
        assert_eq!(bar.handle_click(left_click(col, 0)), Some(ProcessCommands::Info), "col {col}");
    }
}

#[test]
fn clicking_past_the_last_button_returns_nothing() {
    let mut bar = rendered(Rect::new(0, 0, 20, 1), 0);
    for col in 17..20 {
        assert_eq!(bar.handle_click(left_click(col, 0)), None, "col {col}");
    }
}

#[test]
fn a_successful_click_latches_the_command() {
    let mut bar = rendered(Rect::new(0, 0, 20, 1), 0);
    assert_eq!(bar.command, ProcessCommands::Select);

    bar.handle_click(left_click(13, 0));
    assert_eq!(bar.command, ProcessCommands::Info);

    bar.handle_click(left_click(8, 0));
    assert_eq!(bar.command, ProcessCommands::Kill);
}

#[test]
fn a_miss_leaves_the_previous_command_alone() {
    let mut bar = rendered(Rect::new(0, 0, 20, 1), 0);
    bar.handle_click(left_click(13, 0));
    assert_eq!(bar.command, ProcessCommands::Info);

    bar.handle_click(left_click(19, 0)); // past the buttons
    assert_eq!(bar.command, ProcessCommands::Info);
}

#[test]
fn clicks_outside_the_rendered_area_are_ignored() {
    let mut bar = rendered(Rect::new(0, 0, 20, 1), 0);
    assert_eq!(bar.handle_click(left_click(1, 5)), None, "row below");
    assert_eq!(bar.handle_click(left_click(30, 0)), None, "column right");
}

#[test]
fn a_non_zero_origin_shifts_every_button() {
    let mut bar = rendered(Rect::new(10, 3, 20, 1), 0);
    assert_eq!(bar.handle_click(left_click(10, 3)), Some(ProcessCommands::Select));
    assert_eq!(bar.handle_click(left_click(17, 3)), Some(ProcessCommands::Kill));
    assert_eq!(bar.handle_click(left_click(22, 3)), Some(ProcessCommands::Info));
    assert_eq!(bar.handle_click(left_click(9, 3)), None, "left of the bar");
}

#[test]
fn only_a_left_button_press_counts() {
    let mut bar = rendered(Rect::new(0, 0, 20, 1), 0);

    let mut moved = left_click(1, 0);
    moved.kind = MouseEventKind::Moved;
    assert_eq!(bar.handle_click(moved), None);

    let mut right = left_click(1, 0);
    right.kind = MouseEventKind::Down(MouseButton::Right);
    assert_eq!(bar.handle_click(right), None);

    let mut up = left_click(1, 0);
    up.kind = MouseEventKind::Up(MouseButton::Left);
    assert_eq!(bar.handle_click(up), None);
}

// ------------------------------------------------------------------ rendering

#[test]
fn render_draws_every_button_label() {
    let mut bar = ProcessTaskBar::new();
    let text = common::buffer_text(&common::render(40, 1, |f| {
        bar.render(f, Rect::new(0, 0, 30, 1), 0)
    }));

    for label in ["Select", "Kill", "Info"] {
        assert!(text.contains(label), "missing {label} in {text:?}");
    }
}

#[test]
fn labels_are_present_whether_or_not_a_process_is_selected() {
    // selected_pid only changes styling, never which buttons exist.
    let mut bar = ProcessTaskBar::new();
    let none = common::buffer_text(&common::render(40, 1, |f| {
        bar.render(f, Rect::new(0, 0, 30, 1), 0)
    }));
    let some = common::buffer_text(&common::render(40, 1, |f| {
        bar.render(f, Rect::new(0, 0, 30, 1), 4242)
    }));
    assert_eq!(none.trim_end(), some.trim_end());
}

#[test]
fn rendering_into_a_narrow_area_does_not_panic() {
    let mut bar = ProcessTaskBar::new();
    let _ = common::render(4, 1, |f| bar.render(f, Rect::new(0, 0, 4, 1), 0));
}

// ----------------------------------------------------------------- dispatch

#[test]
fn signal_process_is_a_no_op_without_a_selection() {
    let bar = ProcessTaskBar::new();
    let table = ProcessTable::new(); // selected_pid == 0
    let mut info = ProcInfoPopup::new();
    let mut sys = System::new();

    bar.signal_process(&table, &mut info, &mut sys);

    assert!(!info.visable, "nothing should open without a selected pid");
    assert_eq!(info.selected_pid, 0);
}

#[test]
fn the_select_command_never_touches_the_popup() {
    let mut bar = ProcessTaskBar::new();
    bar.command = ProcessCommands::Select;
    let mut table = ProcessTable::new();
    table.selected_pid = 4242;
    let mut info = ProcInfoPopup::new();
    let mut sys = System::new();

    bar.signal_process(&table, &mut info, &mut sys);

    assert!(!info.visable);
    assert_eq!(info.selected_pid, 0);
}

#[test]
fn the_info_command_opens_the_popup_on_the_selected_pid() {
    let mut bar = ProcessTaskBar::new();
    bar.command = ProcessCommands::Info;
    let mut table = ProcessTable::new();
    table.selected_pid = 4242;
    let mut info = ProcInfoPopup::new();
    let mut sys = System::new();

    bar.signal_process(&table, &mut info, &mut sys);

    assert!(info.visable);
    assert_eq!(info.selected_pid, 4242);
}

#[test]
fn the_info_command_toggles_the_popup_shut_again() {
    let mut bar = ProcessTaskBar::new();
    bar.command = ProcessCommands::Info;
    let mut table = ProcessTable::new();
    table.selected_pid = 4242;
    let mut info = ProcInfoPopup::new();
    let mut sys = System::new();

    bar.signal_process(&table, &mut info, &mut sys);
    bar.signal_process(&table, &mut info, &mut sys);

    assert!(!info.visable, "a second Info press should close the popup");
    assert_eq!(info.selected_pid, 4242, "the pid stays latched");
}
