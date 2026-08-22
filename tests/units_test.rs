//! `tools::units` — pure formatting helpers, so these are exhaustive around
//! every unit boundary and the odd edges (zero, NaN, negatives, u64::MAX).

use rust_top::tools::units::{
    format_axis_label, format_bytes, format_kb, format_rate, format_total, GIGABYTE, KILOBYTE,
    MEGABYTE, TERABYTE,
};

#[test]
fn constants_are_binary_multiples() {
    assert_eq!(KILOBYTE, 1024.0);
    assert_eq!(MEGABYTE, KILOBYTE * 1024.0);
    assert_eq!(GIGABYTE, MEGABYTE * 1024.0);
    assert_eq!(TERABYTE, GIGABYTE * 1024.0);
}

// ---------------------------------------------------------------- format_rate

#[test]
fn format_rate_below_one_kilobyte_uses_bytes() {
    assert_eq!(format_rate(0.0), "0b/s");
    assert_eq!(format_rate(1.0), "1b/s");
    assert_eq!(format_rate(512.0), "512b/s");
}

#[test]
fn format_rate_switches_unit_exactly_at_each_boundary() {
    assert_eq!(format_rate(KILOBYTE), "1.0Kb/s");
    assert_eq!(format_rate(KILOBYTE - 1.0), "1023b/s");
    assert_eq!(format_rate(MEGABYTE), "1.0Mb/s");
    assert_eq!(format_rate(MEGABYTE - 1.0), "1024.0Kb/s");
    assert_eq!(format_rate(GIGABYTE), "1.0Gb/s");
    assert_eq!(format_rate(GIGABYTE - 1.0), "1024.0Mb/s");
}

#[test]
fn format_rate_rounds_to_one_decimal() {
    assert_eq!(format_rate(1536.0), "1.5Kb/s");
    assert_eq!(format_rate(1.5 * MEGABYTE), "1.5Mb/s");
    assert_eq!(format_rate(2.25 * GIGABYTE), "2.2Gb/s");
}

#[test]
fn format_rate_has_no_terabyte_tier() {
    // Unlike format_bytes, rates cap out at Gb/s and keep growing the number.
    assert_eq!(format_rate(TERABYTE), "1024.0Gb/s");
}

#[test]
fn format_rate_rounds_sub_kilobyte_values_up_to_1024() {
    // Documents a display quirk: 1023.9 B/s renders as "1024b/s", not "1.0Kb/s".
    assert_eq!(format_rate(1023.9), "1024b/s");
}

#[test]
fn format_rate_handles_negative_and_non_finite_input() {
    assert_eq!(format_rate(-1.0), "-1b/s");
    assert_eq!(format_rate(f64::NAN), "NaNb/s");
    assert_eq!(format_rate(f64::INFINITY), "infGb/s");
}

// --------------------------------------------------------------- format_total

#[test]
fn format_total_below_one_kilobyte_prints_raw_bytes() {
    assert_eq!(format_total(0), "0B");
    assert_eq!(format_total(1), "1B");
    assert_eq!(format_total(1023), "1023B");
}

#[test]
fn format_total_switches_unit_exactly_at_each_boundary() {
    assert_eq!(format_total(1024), "1.0KB");
    assert_eq!(format_total(MEGABYTE as u64), "1.0MB");
    assert_eq!(format_total(MEGABYTE as u64 - 1), "1024.0KB");
    assert_eq!(format_total(GIGABYTE as u64), "1.0GB");
    assert_eq!(format_total(GIGABYTE as u64 - 1), "1024.0MB");
}

#[test]
fn format_total_has_no_terabyte_tier() {
    assert_eq!(format_total(TERABYTE as u64), "1024.0GB");
    assert_eq!(format_total(u64::MAX), "17179869184.0GB");
}

// --------------------------------------------------------------- format_bytes

#[test]
fn format_bytes_uses_two_decimals_and_a_space() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(MEGABYTE as u64), "1.00 MB");
    assert_eq!(format_bytes(GIGABYTE as u64), "1.00 GB");
    assert_eq!(format_bytes(TERABYTE as u64), "1.00 TB");
}

#[test]
fn format_bytes_switches_unit_exactly_at_each_boundary() {
    assert_eq!(format_bytes(MEGABYTE as u64 - 1), "1024.00 KB");
    assert_eq!(format_bytes(GIGABYTE as u64 - 1), "1024.00 MB");
    assert_eq!(format_bytes(TERABYTE as u64 - 1), "1024.00 GB");
}

#[test]
fn format_bytes_saturates_above_a_terabyte() {
    assert_eq!(format_bytes(u64::MAX), "16777216.00 TB");
}

#[test]
fn format_bytes_and_format_total_disagree_on_terabytes() {
    // Same input, different widget: the disk table shows TB, the network
    // graph's totals do not. Pinned so the divergence stays deliberate.
    let one_tb = TERABYTE as u64;
    assert_eq!(format_bytes(one_tb), "1.00 TB");
    assert_eq!(format_total(one_tb), "1024.0GB");
}

// ------------------------------------------------------------------ format_kb

#[test]
fn format_kb_divides_by_1024() {
    assert_eq!(format_kb(0), 0.0);
    assert_eq!(format_kb(1024), 1.0);
    assert_eq!(format_kb(32 * 1024), 32.0);
    assert_eq!(format_kb(512), 0.5);
}

#[test]
fn format_kb_keeps_fractional_precision() {
    assert_eq!(format_kb(1536), 1.5);
    assert!((format_kb(1) - 0.0009765625).abs() < f64::EPSILON);
}

// ---------------------------------------------------------- format_axis_label

#[test]
fn format_axis_label_zero_and_negatives_collapse_to_zero() {
    assert_eq!(format_axis_label(0.0), "0");
    assert_eq!(format_axis_label(-1.0), "0");
    assert_eq!(format_axis_label(-GIGABYTE), "0");
}

#[test]
fn format_axis_label_uses_single_letter_suffixes() {
    assert_eq!(format_axis_label(1.0), "1");
    assert_eq!(format_axis_label(999.0), "999");
    assert_eq!(format_axis_label(KILOBYTE), "1.0K");
    assert_eq!(format_axis_label(MEGABYTE), "1.0M");
    assert_eq!(format_axis_label(GIGABYTE), "1.0G");
}

#[test]
fn format_axis_label_switches_unit_exactly_at_each_boundary() {
    assert_eq!(format_axis_label(KILOBYTE - 1.0), "1023");
    assert_eq!(format_axis_label(MEGABYTE - 1.0), "1024.0K");
    assert_eq!(format_axis_label(GIGABYTE - 1.0), "1024.0M");
    assert_eq!(format_axis_label(TERABYTE), "1024.0G");
}

#[test]
fn format_axis_label_rounds_the_bare_number_tier() {
    assert_eq!(format_axis_label(1023.9), "1024");
    assert_eq!(format_axis_label(0.4), "0");
    assert_eq!(format_axis_label(1536.0), "1.5K");
}
