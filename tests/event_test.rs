//! `event::keystroke_type` — the whole keymap, including the two search modes.
//!
//! `handle_events` itself blocks on terminal input and cannot run headless, so
//! these tests drive the key dispatch it delegates to.
//!
//! Note: the 'k' (kill) binding is only exercised with `selected_pid == 0` so
//! the suite never signals a real process.

mod common;

use common::proc_row;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rust_top::draw::widgets::about_popup::AboutPopUp;
use rust_top::draw::widgets::process_table::{ProcInfoPopup, ProcessTable, SearchState};
use rust_top::draw::widgets::process_taskbar::ProcessTaskBar;
use rust_top::draw::widgets::refresh_ticker::TickButton;
use rust_top::event::keystroke_type;
use std::time::Duration;
use sysinfo::System;

/// Everything `keystroke_type` needs, bundled so tests stay readable.
struct Harness {
    tick: TickButton,
    table: ProcessTable,
    taskbar: ProcessTaskBar,
    info: ProcInfoPopup,
    popup: AboutPopUp,
    sys: System,
}

impl Harness {
    fn new() -> Self {
        Self {
            tick: TickButton::new(Duration::from_millis(2000)),
            table: ProcessTable::new(),
            taskbar: ProcessTaskBar::new(),
            info: ProcInfoPopup::new(),
            popup: AboutPopUp::default(),
            sys: System::new(),
        }
    }

    /// Returns true when the app should quit.
    fn press(&mut self, code: KeyCode) -> bool {
        keystroke_type(
            KeyEvent::new(code, KeyModifiers::NONE),
            &mut self.tick,
            &mut self.table,
            &mut self.taskbar,
            &mut self.info,
            &mut self.popup,
            &mut self.sys,
        )
    }

    fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.press(KeyCode::Char(c));
        }
    }
}

// ----------------------------------------------------------------------- quit

#[test]
fn q_quits() {
    assert!(Harness::new().press(KeyCode::Char('q')));
}

#[test]
fn esc_quits_when_no_search_is_active() {
    assert!(Harness::new().press(KeyCode::Esc));
}

#[test]
fn ordinary_keys_do_not_quit() {
    let mut h = Harness::new();
    for code in [
        KeyCode::Char('p'),
        KeyCode::Char('/'),
        KeyCode::Char('+'),
        KeyCode::Char('-'),
        KeyCode::Char('z'),
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Up,
        KeyCode::Tab,
    ] {
        assert!(!h.press(code), "{code:?} should not quit");
        h.table.search_state = SearchState::NoSearch; // '/' would latch otherwise
    }
}

// ------------------------------------------------------------------ tick rate

#[test]
fn plus_and_equals_both_increase_the_tick_rate() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('+'));
    assert_eq!(h.tick.get_duration(), Duration::from_millis(2100));
    h.press(KeyCode::Char('='));
    assert_eq!(h.tick.get_duration(), Duration::from_millis(2200));
}

#[test]
fn minus_and_underscore_both_decrease_the_tick_rate() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('-'));
    assert_eq!(h.tick.get_duration(), Duration::from_millis(1900));
    h.press(KeyCode::Char('_'));
    assert_eq!(h.tick.get_duration(), Duration::from_millis(1800));
}

#[test]
fn tick_rate_keys_respect_the_clamps() {
    let mut h = Harness::new();
    for _ in 0..100 {
        h.press(KeyCode::Char('+'));
    }
    assert_eq!(h.tick.get_duration(), Duration::from_millis(5000));
    for _ in 0..100 {
        h.press(KeyCode::Char('-'));
    }
    assert_eq!(h.tick.get_duration(), Duration::from_millis(1000));
}

// ---------------------------------------------------------------- about popup

#[test]
fn p_toggles_the_about_popup() {
    let mut h = Harness::new();
    assert!(!h.popup.visable);
    h.press(KeyCode::Char('p'));
    assert!(h.popup.visable);
    h.press(KeyCode::Char('p'));
    assert!(!h.popup.visable);
}

// ----------------------------------------------------------------- info popup

#[test]
fn i_does_nothing_without_a_selected_process() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('i'));
    assert!(!h.info.visable);
    assert_eq!(h.info.selected_pid, 0);
}

#[test]
fn i_toggles_the_info_popup_for_the_selected_process() {
    let mut h = Harness::new();
    h.table.selected_pid = 4242;

    h.press(KeyCode::Char('i'));
    assert!(h.info.visable);
    assert_eq!(h.info.selected_pid, 4242);

    h.press(KeyCode::Char('i'));
    assert!(!h.info.visable, "a second press closes it");
}

// ----------------------------------------------------------------------- kill

#[test]
fn k_is_inert_without_a_selection() {
    // Guarded so the suite never signals a live process.
    let mut h = Harness::new();
    h.table.proc_table = vec![proc_row(1, "a", "a", 1.0, 0.0)];

    assert!(!h.press(KeyCode::Char('k')));
    assert_eq!(h.table.proc_table.len(), 1, "nothing should be removed");
    assert_eq!(h.table.selected_pid, 0);
}

// --------------------------------------------------------------- search entry

#[test]
fn slash_enters_the_typing_state() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('/'));
    assert!(h.table.search_state == SearchState::Searching);
    assert!(h.table.is_filter_input_active());
}

#[test]
fn slash_clears_any_previous_query() {
    let mut h = Harness::new();
    h.table.search_input = "stale".into();
    h.press(KeyCode::Char('/'));
    assert!(h.table.search_input.is_empty());
}

#[test]
fn typing_while_searching_builds_the_query() {
    let mut h = Harness::new();
    h.table.proc_table = vec![
        proc_row(1, "firefox", "firefox", 1.0, 0.0),
        proc_row(2, "bash", "bash", 1.0, 0.0),
    ];
    h.press(KeyCode::Char('/'));
    h.type_str("f");

    assert_eq!(h.table.search_input, "f");
    assert_eq!(h.table.filtered_table.len(), 1, "filter applied on each keystroke");
    assert_eq!(h.table.filtered_table[0].pid, 1);
}

#[test]
fn quit_keys_are_captured_by_the_search_input() {
    // While typing, 'q' is a character, not a quit command.
    let mut h = Harness::new();
    h.press(KeyCode::Char('/'));
    assert!(!h.press(KeyCode::Char('q')));
    assert_eq!(h.table.search_input, "q");
}

#[test]
fn backspace_removes_the_last_character() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('/'));
    h.type_str("abc");
    assert_eq!(h.table.search_input, "abc");

    h.press(KeyCode::Backspace);
    assert_eq!(h.table.search_input, "ab");
}

#[test]
fn backspace_on_an_empty_query_is_harmless() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('/'));
    h.press(KeyCode::Backspace);
    assert!(h.table.search_input.is_empty());
    assert!(h.table.search_state == SearchState::Searching);
}

#[test]
fn enter_commits_the_filter() {
    let mut h = Harness::new();
    h.press(KeyCode::Char('/'));
    h.type_str("ba");
    h.press(KeyCode::Enter);

    assert!(h.table.search_state == SearchState::FilterApplied);
    assert!(h.table.is_searching());
    assert!(!h.table.is_filter_input_active(), "the input box closes on commit");
}

#[test]
fn typing_a_second_character_wipes_the_results() {
    // BUG (pinned): apply_filter() clears proc_table after each keystroke, so
    // the second character filters an already-empty source. Typing anything
    // longer than one letter always yields an empty result list.
    let mut h = Harness::new();
    h.table.proc_table = vec![
        proc_row(1, "firefox", "firefox", 1.0, 0.0),
        proc_row(2, "bash", "bash", 1.0, 0.0),
    ];
    h.press(KeyCode::Char('/'));

    h.type_str("f");
    assert_eq!(h.table.filtered_table.len(), 1, "the first keystroke works");

    h.type_str("i");
    assert_eq!(h.table.search_input, "fi");
    assert!(
        h.table.filtered_table.is_empty(),
        "the second keystroke filters an emptied proc_table"
    );
}

#[test]
fn backspace_also_re_filters_an_emptied_table() {
    // Same root cause: backspace calls apply_filter() against a drained source.
    let mut h = Harness::new();
    h.table.proc_table = vec![proc_row(1, "firefox", "firefox", 1.0, 0.0)];
    h.press(KeyCode::Char('/'));
    h.type_str("f");
    assert_eq!(h.table.filtered_table.len(), 1);

    h.press(KeyCode::Backspace);
    assert!(h.table.search_input.is_empty());
    assert!(
        h.table.filtered_table.is_empty(),
        "the row is not restored until the next refresh"
    );
}

#[test]
fn esc_while_typing_cancels_the_search() {
    let mut h = Harness::new();
    h.table.proc_table = vec![proc_row(1, "firefox", "firefox", 1.0, 0.0)];
    h.press(KeyCode::Char('/'));
    h.type_str("f");
    assert!(!h.table.filtered_table.is_empty());

    assert!(!h.press(KeyCode::Esc), "esc must cancel the search, not quit");
    assert!(h.table.search_state == SearchState::NoSearch);
    assert!(h.table.search_input.is_empty());
    assert!(h.table.filtered_table.is_empty());
}

// ------------------------------------------------------------- filter applied

#[test]
fn esc_clears_a_committed_filter() {
    let mut h = Harness::new();
    h.table.search_state = SearchState::FilterApplied;
    h.table.search_input = "fire".into();
    h.table.filtered_table = vec![proc_row(1, "firefox", "firefox", 1.0, 0.0)];

    assert!(!h.press(KeyCode::Esc), "esc must clear the filter, not quit");
    assert!(h.table.search_state == SearchState::NoSearch);
    assert!(h.table.search_input.is_empty());
    assert!(h.table.filtered_table.is_empty());
}

#[test]
fn every_other_key_is_swallowed_while_a_filter_is_applied() {
    let mut h = Harness::new();
    h.table.search_state = SearchState::FilterApplied;
    h.table.search_input = "fire".into();
    let before = h.tick.get_duration();

    for code in [
        KeyCode::Char('q'),
        KeyCode::Char('p'),
        KeyCode::Char('+'),
        KeyCode::Char('-'),
        KeyCode::Char('/'),
        KeyCode::Char('x'),
        KeyCode::Enter,
        KeyCode::Backspace,
    ] {
        assert!(!h.press(code), "{code:?} should be swallowed");
    }

    assert!(h.table.search_state == SearchState::FilterApplied, "state must not move");
    assert_eq!(h.table.search_input, "fire", "query must not change");
    assert_eq!(h.tick.get_duration(), before, "tick rate must not change");
    assert!(!h.popup.visable, "popup must not open");
}

// ------------------------------------------------------------ full round trip

#[test]
fn search_lifecycle_returns_to_the_starting_state() {
    let mut h = Harness::new();
    h.table.proc_table = vec![
        proc_row(1, "firefox", "firefox", 1.0, 0.0),
        proc_row(2, "bash", "bash", 1.0, 0.0),
    ];

    h.press(KeyCode::Char('/'));
    h.type_str("fire");
    h.press(KeyCode::Enter);
    assert!(h.table.search_state == SearchState::FilterApplied);

    h.press(KeyCode::Esc);
    assert!(h.table.search_state == SearchState::NoSearch);
    assert!(!h.table.is_searching());
    assert!(h.table.search_input.is_empty());
    assert!(h.table.filtered_table.is_empty());

    // Normal bindings work again.
    h.press(KeyCode::Char('p'));
    assert!(h.popup.visable);
}

#[test]
fn unhandled_keys_change_nothing() {
    let mut h = Harness::new();
    let tick = h.tick.get_duration();

    for code in [KeyCode::Up, KeyCode::Down, KeyCode::Tab, KeyCode::F(5), KeyCode::Home] {
        assert!(!h.press(code));
    }

    assert_eq!(h.tick.get_duration(), tick);
    assert!(!h.popup.visable);
    assert!(!h.info.visable);
    assert!(h.table.search_state == SearchState::NoSearch);
}
