//! `collection::TopCollection` — the aggregate the main loop drives. These are
//! the slowest tests in the suite because each one builds a full `System`.

use rust_top::collection::TopCollection;
use rust_top::data::cpu::CpuInfo;

fn lengths_match(c: &TopCollection) -> bool {
    let d = &c.disk_data;
    let n = d.disk_name.len();
    d.filesytem.len() == n
        && d.mount.len() == n
        && d.total.len() == n
        && d.available.len() == n
        && d.curr_read.len() == n
        && d.curr_write.len() == n
}

#[test]
fn new_collection_is_fully_populated() {
    let mut c = TopCollection::new();

    assert!(!c.cpu_model_lines.spans.is_empty(), "cpu model line is empty");
    assert_eq!(c.mem_lines.len(), 2, "memory panel is two lines");

    let cores = CpuInfo::num_cores(&mut c.sys);
    assert!(cores > 0);
    assert_eq!(c.core_graph.cores.len(), cores, "one graph entry per core");
}

#[test]
fn new_collection_starts_with_zeroed_core_usage() {
    let c = TopCollection::new();
    assert!(
        c.core_graph.cores.values().all(|v| *v == 0.0),
        "core usages should start at zero until the first update"
    );
}

#[test]
fn new_collection_has_no_disk_or_process_data_yet() {
    let c = TopCollection::new();
    assert!(c.disk_data.disk_name.is_empty(), "disks are collected in update_data");
    assert!(c.process_tree.proc_table.is_empty());
    assert_eq!(c.process_tree.selected_pid, 0);
}

#[test]
fn one_full_cycle_populates_disks_and_processes() {
    let mut c = TopCollection::new();
    c.refresh_timer();
    c.update_data();

    assert!(lengths_match(&c), "disk vectors fell out of sync");
    assert!(
        !c.process_tree.proc_table.is_empty(),
        "the test process itself should appear"
    );
}

#[test]
fn update_data_keeps_one_graph_entry_per_core() {
    let mut c = TopCollection::new();
    let cores = CpuInfo::num_cores(&mut c.sys);

    c.refresh_timer();
    c.update_data();

    assert_eq!(c.core_graph.cores.len(), cores, "cores must not accumulate");
    assert!(
        c.core_graph.cores.values().all(|v| v.is_finite()),
        "a non-finite core usage would break the bar maths"
    );
}

#[test]
fn repeated_cycles_do_not_grow_any_collection() {
    let mut c = TopCollection::new();
    let cores = CpuInfo::num_cores(&mut c.sys);

    c.refresh_timer();
    c.update_data();
    let disks_after_first = c.disk_data.disk_name.len();
    let voltages_after_first = c.voltage_bar.voltages.voltage_read_points.len();

    for _ in 0..3 {
        c.refresh_timer();
        c.update_data();
    }

    assert_eq!(c.core_graph.cores.len(), cores);
    assert_eq!(
        c.disk_data.disk_name.len(),
        disks_after_first,
        "disk rows must be replaced, not appended"
    );
    assert!(lengths_match(&c));
    assert_eq!(c.mem_lines.len(), 2);
    assert_eq!(
        c.voltage_bar.voltages.voltage_read_points.len(),
        voltages_after_first,
        "voltage readings must be replaced, not appended"
    );
}

#[test]
fn every_process_row_stays_within_range_across_cycles() {
    let mut c = TopCollection::new();
    for _ in 0..2 {
        c.refresh_timer();
        c.update_data();
    }

    for p in &c.process_tree.proc_table {
        assert!(p.cpu_usage_percent.is_finite(), "pid {} cpu not finite", p.pid);
        assert!(
            (0.0..=100.5).contains(&p.mem_usage_percent),
            "pid {} reported {}% memory",
            p.pid,
            p.mem_usage_percent
        );
    }
}

#[test]
fn update_data_without_a_prior_refresh_timer_still_works() {
    // The main loop always pairs them, but update_data must not depend on it.
    let mut c = TopCollection::new();
    c.update_data();
    assert!(lengths_match(&c));
    assert!(!c.process_tree.proc_table.is_empty());
}
