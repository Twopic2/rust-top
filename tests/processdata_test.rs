//! `processes::processdata` — the /proc scraper, the uid→label mapping and the
//! per-process collection pass.

mod common;

use rust_top::processes::processdata::{pid_io_results, uid_to_user, CollectProcessData, Io};

// ----------------------------------------------------------------- uid_to_user

#[test]
fn uid_zero_is_root() {
    assert_eq!(uid_to_user(Some(0)), "Root User");
}

#[test]
fn uid_1000_is_the_base_user() {
    assert_eq!(uid_to_user(Some(1000)), "Base User");
}

#[test]
fn missing_uid_gets_the_unknown_label() {
    assert_eq!(uid_to_user(None), "Idk what's up");
}

#[test]
fn any_other_uid_falls_through_to_other() {
    for uid in [1, 42, 999, 1001, 65534, u32::MAX] {
        assert_eq!(uid_to_user(Some(uid)), "Other User", "uid {uid}");
    }
}

// ------------------------------------------------------------------------- Io

#[test]
fn io_defaults_to_zero() {
    let io = Io::default();
    assert_eq!(io.read_bytes, 0);
    assert_eq!(io.write_bytes, 0);
}

#[test]
fn io_is_cloneable() {
    let io = Io { read_bytes: 7, write_bytes: 9 };
    let copy = io.clone();
    assert_eq!(copy.read_bytes, 7);
    assert_eq!(copy.write_bytes, 9);
}

// -------------------------------------------------------------- pid_io_results

#[cfg(target_os = "linux")]
#[test]
fn reads_io_counters_for_the_current_process() {
    let io = pid_io_results(std::process::id()).expect("/proc/self/io should be readable");
    // Values are cumulative counters; we can only assert they parsed.
    assert!(io.read_bytes < u64::MAX);
    assert!(io.write_bytes < u64::MAX);
}

#[cfg(target_os = "linux")]
#[test]
fn counters_are_monotonic_for_a_live_process() {
    let pid = std::process::id();
    let before = pid_io_results(pid).expect("first read");
    // Force some real IO so the counters have a chance to move.
    let _ = std::fs::read_to_string("/proc/self/status");
    let after = pid_io_results(pid).expect("second read");

    assert!(after.read_bytes >= before.read_bytes, "read_bytes went backwards");
    assert!(after.write_bytes >= before.write_bytes, "write_bytes went backwards");
}

#[test]
fn a_pid_that_cannot_exist_returns_an_error() {
    assert!(
        pid_io_results(u32::MAX).is_err(),
        "opening /proc/{}/io must fail",
        u32::MAX
    );
}

#[test]
fn pid_zero_returns_an_error() {
    assert!(pid_io_results(0).is_err(), "there is no /proc/0");
}

// -------------------------------------------------- CollectProcessData::default

#[test]
fn default_process_row_is_zeroed() {
    let d = CollectProcessData::default();
    assert_eq!(d.pid, 0);
    assert!(d.command.is_empty());
    assert!(d.program.is_empty());
    assert_eq!(d.cpu_usage_percent, 0.0);
    assert_eq!(d.mem_usage_percent, 0.0);
    assert!(d.disk_io.is_none());
    assert!(d.uid.is_none());
    assert_eq!(d.user, "");
}

// ----------------------------------------------------------------- process_data

#[test]
fn process_data_returns_a_non_empty_snapshot() {
    let mut sys = common::loaded_system();
    let rows = CollectProcessData::default().process_data(&mut sys);
    assert!(!rows.is_empty(), "the running test process should appear");
}

#[test]
fn every_returned_row_passes_the_activity_filter() {
    // process_data only keeps rows with cpu > 0 OR mem > 0.
    let mut sys = common::loaded_system();
    for p in CollectProcessData::default().process_data(&mut sys) {
        assert!(
            p.cpu_usage_percent > 0.0 || p.mem_usage_percent > 0.0,
            "pid {} slipped past the activity filter ({}% cpu, {}% mem)",
            p.pid,
            p.cpu_usage_percent,
            p.mem_usage_percent
        );
    }
}

#[test]
fn percentages_are_finite_and_in_range() {
    let mut sys = common::loaded_system();
    for p in CollectProcessData::default().process_data(&mut sys) {
        assert!(p.cpu_usage_percent.is_finite(), "pid {} cpu not finite", p.pid);
        assert!(p.mem_usage_percent.is_finite(), "pid {} mem not finite", p.pid);
        assert!(
            (0.0..=100.5).contains(&p.mem_usage_percent),
            "pid {} reported {}% memory",
            p.pid,
            p.mem_usage_percent
        );
    }
}

#[test]
fn snapshot_never_exceeds_the_number_of_live_processes() {
    let mut sys = common::loaded_system();
    let total = sys.processes().len();
    let rows = CollectProcessData::default().process_data(&mut sys);
    assert!(rows.len() <= total, "{} rows from {total} processes", rows.len());
}

#[test]
fn every_row_carries_a_user_label() {
    let mut sys = common::loaded_system();
    for p in CollectProcessData::default().process_data(&mut sys) {
        assert!(
            ["Root User", "Base User", "Other User", "Idk what's up"].contains(&p.user),
            "pid {} had an unexpected user label {:?}",
            p.pid,
            p.user
        );
        // The label must agree with the raw uid it was derived from.
        assert_eq!(p.user, uid_to_user(p.uid), "pid {} label/uid mismatch", p.pid);
    }
}

#[test]
fn pids_in_a_single_snapshot_are_unique() {
    let mut sys = common::loaded_system();
    let rows = CollectProcessData::default().process_data(&mut sys);
    let mut seen: Vec<u32> = common::pids(&rows);
    seen.sort_unstable();
    let before = seen.len();
    seen.dedup();
    assert_eq!(before, seen.len(), "process_data emitted duplicate pids");
}

#[test]
fn repeated_collection_stays_stable() {
    let mut sys = common::loaded_system();
    let mut collector = CollectProcessData::default();
    let first = collector.process_data(&mut sys).len();
    let second = collector.process_data(&mut sys).len();
    assert!(first > 0 && second > 0);
    // The collector holds no state, so a second pass over the same System
    // snapshot must produce the same number of rows.
    assert_eq!(first, second);
}

#[test]
fn rows_are_cloneable_for_the_filter_path() {
    // ProcessTable::apply_filter clones matching rows; make sure the derive holds.
    let row = common::proc_row_with_io(42, "prog", "prog --flag", 1.5, 2.5, 10, 20);
    let copy = row.clone();
    assert_eq!(copy.pid, 42);
    assert_eq!(copy.program, "prog");
    assert_eq!(copy.command, "prog --flag");
    assert_eq!(copy.disk_io.unwrap().read_bytes, 10);
}
