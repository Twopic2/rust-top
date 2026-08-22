//! `data::mem` — the two summary lines rendered into the Memory panel.

mod common;

use rust_top::data::mem::MemInfo;

#[test]
fn display_memory_returns_exactly_two_lines() {
    let mut s = common::loaded_system();
    assert_eq!(MemInfo::display_memory(&mut s).len(), 2);
}

#[test]
fn first_line_covers_totals_and_second_covers_usage() {
    let mut s = common::loaded_system();
    let lines = MemInfo::display_memory(&mut s);

    assert!(lines[0].starts_with("Total: "), "got {:?}", lines[0]);
    assert!(lines[0].contains("Total Swap: "), "got {:?}", lines[0]);
    assert!(lines[1].starts_with("Used: "), "got {:?}", lines[1]);
    assert!(lines[1].contains("Used Swap: ("), "got {:?}", lines[1]);
}

#[test]
fn every_value_is_rendered_in_gigabytes_with_two_decimals() {
    let mut s = common::loaded_system();
    for line in MemInfo::display_memory(&mut s) {
        assert_eq!(line.matches(" GB").count(), 2, "expected two GB values in {line:?}");
        for token in line.split_whitespace() {
            if let Some(num) = token.strip_prefix('(') {
                assert_decimals(num);
            }
        }
    }
}

fn assert_decimals(token: &str) {
    if let Some((_, frac)) = token.split_once('.') {
        assert_eq!(frac.len(), 2, "expected 2 decimal places in {token:?}");
    }
}

#[test]
fn totals_are_not_smaller_than_used() {
    // Parsed straight back out of the rendered strings, since that is the only
    // thing display_memory exposes.
    let mut s = common::loaded_system();
    let lines = MemInfo::display_memory(&mut s);

    let total = first_number(&lines[0]);
    let used = first_number(&lines[1]);
    assert!(
        used <= total + 0.01,
        "used memory {used} GB exceeded total {total} GB"
    );
}

fn first_number(line: &str) -> f64 {
    line.split_whitespace()
        .find_map(|t| t.trim_matches(|c: char| !c.is_ascii_digit() && c != '.').parse::<f64>().ok())
        .expect("a number in the line")
}

#[test]
fn display_memory_is_stable_across_repeated_calls() {
    let mut s = common::loaded_system();
    let a = MemInfo::display_memory(&mut s);
    let b = MemInfo::display_memory(&mut s);
    // Same System, no refresh in between: totals must not move.
    assert_eq!(a[0], b[0]);
}
