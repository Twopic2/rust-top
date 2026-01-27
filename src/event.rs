use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use crate::draw::misc::{TickButton, TickCounter};

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

fn keystroke_type(event: KeyEvent, tick_button: &mut TickButton) -> bool {
    match event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            true
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            tick_button.increment();
            false
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            tick_button.decrement();
            false
        }
        _ => false
    }
}

pub fn handle_events(tick_button: &mut TickButton) -> io::Result<bool> {
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
                    if keystroke_type(key, tick_button) {
                        return Ok(true);
                    }
                }
                TopEvent::MouseInput(mouse_event) => {
                    mouse_ticker_click(mouse_event, tick_button);
                }
            }
        }
    }
    Ok(false)
}
