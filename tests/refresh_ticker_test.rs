//! `draw::widgets::refresh_ticker` — tick-rate clamping and the +/- hit boxes.
//! Bounds are MIN 1000ms, MAX 5000ms, step 100ms.

mod common;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use rust_top::draw::widgets::refresh_ticker::{TickButton, TickCounter};
use std::time::Duration;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn left_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

/// Render the button once so `duration_click` has a hit box to test against.
fn rendered(duration: Duration, area: Rect) -> TickButton {
    let mut b = TickButton::new(duration);
    let _ = common::render(40, 3, |f| b.render(f, area));
    b
}

// ------------------------------------------------------------------- stepping

#[test]
fn new_stores_the_duration_verbatim() {
    assert_eq!(TickButton::new(ms(2000)).get_duration(), ms(2000));
}

#[test]
fn increment_adds_one_step() {
    let mut b = TickButton::new(ms(2000));
    b.increment();
    assert_eq!(b.get_duration(), ms(2100));
}

#[test]
fn decrement_subtracts_one_step() {
    let mut b = TickButton::new(ms(2000));
    b.decrement();
    assert_eq!(b.get_duration(), ms(1900));
}

#[test]
fn increment_stops_at_the_maximum() {
    let mut b = TickButton::new(ms(5000));
    b.increment();
    assert_eq!(b.get_duration(), ms(5000));
}

#[test]
fn decrement_stops_at_the_minimum() {
    let mut b = TickButton::new(ms(1000));
    b.decrement();
    assert_eq!(b.get_duration(), ms(1000));
}

#[test]
fn a_step_that_would_overshoot_is_refused_rather_than_clamped() {
    // 4950 + 100 = 5050 > MAX, so the value stays at 4950 instead of snapping to 5000.
    let mut b = TickButton::new(ms(4950));
    b.increment();
    assert_eq!(b.get_duration(), ms(4950));

    // Same on the way down: 1050 - 100 = 950 < MIN.
    let mut b = TickButton::new(ms(1050));
    b.decrement();
    assert_eq!(b.get_duration(), ms(1050));
}

#[test]
fn repeated_increments_saturate_at_the_maximum() {
    let mut b = TickButton::new(ms(1000));
    for _ in 0..100 {
        b.increment();
    }
    assert_eq!(b.get_duration(), ms(5000));
}

#[test]
fn repeated_decrements_settle_on_the_minimum() {
    let mut b = TickButton::new(ms(5000));
    for _ in 0..100 {
        b.decrement();
    }
    assert_eq!(b.get_duration(), ms(1000));
}

#[test]
fn increment_and_decrement_round_trip() {
    let mut b = TickButton::new(ms(3000));
    b.increment();
    b.decrement();
    assert_eq!(b.get_duration(), ms(3000));
}

#[test]
fn a_below_minimum_start_can_climb_but_is_never_snapped_up() {
    // new() does not clamp, so a caller can construct an out-of-range button.
    let mut b = TickButton::new(ms(0));
    b.decrement();
    assert_eq!(b.get_duration(), ms(0), "decrement refuses to go below MIN");
    b.increment();
    assert_eq!(b.get_duration(), ms(100), "increment still steps normally");
}

// ------------------------------------------------------------------ hit boxes

#[test]
fn clicking_before_the_first_render_does_nothing() {
    let b = TickButton::new(ms(2000));
    assert_eq!(b.duration_click(left_click(1, 0)), TickCounter::None);
}

#[test]
fn the_leftmost_three_columns_decrement() {
    let b = rendered(ms(2000), Rect::new(0, 0, 14, 1));
    for col in 0..3 {
        assert_eq!(
            b.duration_click(left_click(col, 0)),
            TickCounter::Decrement,
            "column {col}"
        );
    }
}

#[test]
fn the_trailing_columns_increment() {
    // " - 2000ms  + " is 13 chars, so the increment zone starts at column 10.
    let b = rendered(ms(2000), Rect::new(0, 0, 14, 1));
    for col in 10..14 {
        assert_eq!(
            b.duration_click(left_click(col, 0)),
            TickCounter::Increment,
            "column {col}"
        );
    }
}

#[test]
fn the_middle_label_is_inert() {
    let b = rendered(ms(2000), Rect::new(0, 0, 14, 1));
    for col in 3..10 {
        assert_eq!(
            b.duration_click(left_click(col, 0)),
            TickCounter::None,
            "column {col}"
        );
    }
}

#[test]
fn the_increment_zone_moves_with_the_label_width() {
    // A three-digit duration renders one char shorter, shifting the zone left.
    let b = rendered(ms(900), Rect::new(0, 0, 14, 1));
    assert_eq!(b.duration_click(left_click(9, 0)), TickCounter::Increment);
    assert_eq!(b.duration_click(left_click(8, 0)), TickCounter::None);
}

#[test]
fn the_hit_box_is_computed_from_a_narrower_string_than_is_drawn() {
    // BUG (pinned): render draws " - " + " {ms}ms " + " + " (14 cols at 2000ms)
    // while duration_click measures " - {ms}ms  + " (13 cols). The increment
    // zone therefore starts one column left of the drawn "+".
    let b = rendered(ms(2000), Rect::new(0, 0, 14, 1));
    assert_eq!(
        b.duration_click(left_click(10, 0)),
        TickCounter::Increment,
        "column 10 renders as blank padding but still increments"
    );
}

#[test]
fn clicks_outside_the_rendered_row_are_ignored() {
    let b = rendered(ms(2000), Rect::new(0, 0, 14, 1));
    assert_eq!(b.duration_click(left_click(1, 1)), TickCounter::None, "row below");
    assert_eq!(b.duration_click(left_click(30, 0)), TickCounter::None, "column right");
}

#[test]
fn a_non_zero_origin_shifts_both_zones() {
    let b = rendered(ms(2000), Rect::new(10, 2, 14, 1));
    assert_eq!(b.duration_click(left_click(10, 2)), TickCounter::Decrement);
    assert_eq!(b.duration_click(left_click(20, 2)), TickCounter::Increment);
    assert_eq!(b.duration_click(left_click(9, 2)), TickCounter::None, "left of the widget");
    assert_eq!(b.duration_click(left_click(15, 2)), TickCounter::None, "the label");
}

#[test]
fn only_a_left_button_press_counts() {
    let b = rendered(ms(2000), Rect::new(0, 0, 14, 1));

    let mut moved = left_click(1, 0);
    moved.kind = MouseEventKind::Moved;
    assert_eq!(b.duration_click(moved), TickCounter::None);

    let mut right = left_click(1, 0);
    right.kind = MouseEventKind::Down(MouseButton::Right);
    assert_eq!(b.duration_click(right), TickCounter::None);

    let mut up = left_click(1, 0);
    up.kind = MouseEventKind::Up(MouseButton::Left);
    assert_eq!(b.duration_click(up), TickCounter::None);
}

// ------------------------------------------------------------------ rendering

#[test]
fn render_shows_the_current_rate_in_milliseconds() {
    let mut b = TickButton::new(ms(2500));
    let buf = common::render(40, 1, |f| b.render(f, Rect::new(0, 0, 20, 1)));
    let text = common::buffer_text(&buf);

    assert!(text.contains("2500ms"), "got {text:?}");
    assert!(text.contains('-') && text.contains('+'), "got {text:?}");
}

#[test]
fn render_reflects_a_changed_rate() {
    let mut b = TickButton::new(ms(2000));
    b.increment();
    let text = common::buffer_text(&common::render(40, 1, |f| b.render(f, Rect::new(0, 0, 20, 1))));
    assert!(text.contains("2100ms"), "got {text:?}");
}
