//! `data::cpu` — reads the real CPU, so assertions are on shape and invariants
//! rather than on specific values.

mod common;

use rust_top::data::cpu::CpuInfo;
use sysinfo::System;

fn sys() -> System {
    common::loaded_system()
}

#[test]
fn num_cores_is_positive() {
    let mut s = sys();
    assert!(CpuInfo::num_cores(&mut s) > 0, "machine must report >=1 core");
}

#[test]
fn num_cores_is_stable_across_calls() {
    let mut s = sys();
    let first = CpuInfo::num_cores(&mut s);
    CpuInfo::set_refresh_timer(&mut s);
    assert_eq!(CpuInfo::num_cores(&mut s), first);
}

#[test]
fn display_cores_returns_one_blank_spacer_then_one_line_per_core() {
    let mut s = sys();
    let cores = CpuInfo::display_cores(&mut s).expect("cpus present");
    let expected = CpuInfo::num_cores(&mut s) + 1;

    assert_eq!(cores.len(), expected);
    assert_eq!(cores[0], "", "first entry is a deliberate blank spacer");
    for (i, line) in cores[1..].iter().enumerate() {
        assert!(
            line.starts_with(&format!("Core {}: ", i)),
            "unexpected core line: {line:?}"
        );
        assert!(line.ends_with('%'), "core line must end in a percent: {line:?}");
    }
}

#[test]
fn display_cpu_frequency_is_reported() {
    let mut s = sys();
    let freq = CpuInfo::display_cpu_frequency(&mut s);
    assert!(freq.is_some(), "at least one cpu should expose a frequency");
}

#[test]
fn display_cpu_model_is_prefixed_with_brand() {
    let mut s = sys();
    let model = CpuInfo::display_cpu_model(&mut s).expect("cpu brand present");
    assert!(model.starts_with("Brand: "), "got {model:?}");
    assert!(model.len() > "Brand: ".len(), "brand string should not be empty");
}

#[test]
fn get_core_usages_has_one_entry_per_core() {
    let mut s = sys();
    let usages = CpuInfo::get_core_usages(&mut s);
    assert_eq!(usages.len(), CpuInfo::num_cores(&mut s));
}

#[test]
fn get_core_usages_are_finite_percentages() {
    let mut s = sys();
    CpuInfo::set_refresh_timer(&mut s);
    for (i, u) in CpuInfo::get_core_usages(&mut s).iter().enumerate() {
        assert!(u.is_finite(), "core {i} usage was not finite: {u}");
        assert!(
            (0.0..=100.5).contains(u),
            "core {i} usage {u} outside 0..100"
        );
    }
}

#[test]
fn set_refresh_timer_keeps_the_core_count_stable() {
    let mut s = sys();
    let before = CpuInfo::get_core_usages(&mut s).len();
    for _ in 0..3 {
        CpuInfo::set_refresh_timer(&mut s);
    }
    assert_eq!(CpuInfo::get_core_usages(&mut s).len(), before);
}

#[test]
fn set_refresh_timer_populates_process_and_memory_data() {
    // set_refresh_timer is the only thing the main loop calls before reading
    // memory and processes, so it has to leave both usable.
    let mut s = System::new();
    CpuInfo::set_refresh_timer(&mut s);
    assert!(s.total_memory() > 0, "memory not refreshed");
    assert!(!s.processes().is_empty(), "processes not refreshed");
}

#[cfg(not(target_os = "macos"))]
#[test]
fn display_cpu_cache_reports_known_levels_with_positive_sizes() {
    let cache = CpuInfo::display_cpu_cache();
    for (label, kb) in &cache {
        assert!(
            matches!(*label, "L1" | "L2"),
            "unexpected cache level {label:?}"
        );
        assert!(*kb > 0.0, "{label} cache size should be positive, got {kb}");
    }
    assert!(cache.len() <= 2, "only L1 and L2 are collected");
}
