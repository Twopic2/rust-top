use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use sysinfo::System;
use crate::draw::widgets::refresh_ticker::{TickButton, TickCounter};
use crate::draw::widgets::process_table::{ProcessTable, SearchState};
use crate::draw::widgets::process_taskbar::{ProcessTaskBar, ProcessCommands};
use crate::draw::widgets::about_popup::AboutPopUp;

#[derive(Debug)]
pub enum TopEvent {
    KeyInput(KeyEvent),
    MouseInput(MouseEvent),
}

pub fn poll_event(timeout: Duration) -> io::Result<Option<TopEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Mouse(mouse) => {
                Ok(Some(TopEvent::MouseInput(mouse)))
            }
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                Ok(Some(TopEvent::KeyInput(key)))
            }
            
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn mouse_ticker_click(event: MouseEvent, tick_button: &mut TickButton) {
    match tick_button.duration_click(event) {
        TickCounter::Increment => tick_button.increment(),
        TickCounter::Decrement => tick_button.decrement(),
        TickCounter::None => {}
    }
}

fn keystroke_type(event: KeyEvent, tick_button: &mut TickButton, process_widget: &mut ProcessTable, 
    taskbar: &mut ProcessTaskBar, popup: &mut AboutPopUp, sys: &mut System) -> bool {
    if process_widget.search_state == SearchState::FilterApplied {
        if event.code == KeyCode::Esc {
            process_widget.search_input.clear();
            process_widget.filtered_table.clear();
            process_widget.search_state = SearchState::NoSearch;
        }
        return false;
    }

    if process_widget.is_filter_input_active() {
        match event.code {
            KeyCode::Enter => {
                process_widget.search_state = SearchState::FilterApplied;
            }
            KeyCode::Esc => {
                process_widget.search_input.clear();
                process_widget.filtered_table.clear();
                process_widget.search_state = SearchState::NoSearch;
            }
            KeyCode::Backspace => {
                process_widget.search_input.pop();
                process_widget.apply_filter();
            }
            KeyCode::Char(c) => {
                process_widget.search_input.push(c);
                process_widget.apply_filter();
            }
            _ => {}
        }
        return false;
    }

    match event.code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('k') => {
            let pid = process_widget.selected_pid;
            if pid != 0 {
                taskbar.signal_process(process_widget, sys);
                process_widget.delete_table_entry(pid);
            }
            false
        } 
        KeyCode::Char('p') => {
            popup.visable = !popup.visable;
            false
        }
        KeyCode::Char('/') => {
            process_widget.search_state = SearchState::Searching;
            process_widget.search_input.clear();
            false
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            tick_button.increment();
            false
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            tick_button.decrement();
            false
        }
        _ => false,
    }
}

pub fn handle_events(tick_button: &mut TickButton, process_widget: &mut ProcessTable, taskbar: &mut ProcessTaskBar, popup: &mut AboutPopUp, sys: &mut System) -> io::Result<bool> {
    let tick_rate = tick_button.get_duration();
    let deadline = Instant::now() + tick_rate;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        if let Some(event) = poll_event(remaining)? {
            match event {
                TopEvent::KeyInput(key) => {
                    if keystroke_type(key, tick_button, process_widget, taskbar, popup, sys) {
                        return Ok(true);
                    }
                    break;
                }
                TopEvent::MouseInput(mouse_event) => {
                    mouse_ticker_click(mouse_event, tick_button);
                    if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
                        process_widget.handle_click(mouse_event.column, mouse_event.row);
                        if let Some(ProcessCommands::Kill) = taskbar.handle_click(mouse_event) {
                            let pid = process_widget.selected_pid;
                            if pid != 0 {
                                taskbar.signal_process(process_widget, sys);
                                process_widget.delete_table_entry(pid);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    Ok(false)
}
