//! `data::disk` — DiskData keeps seven parallel vectors that disk_table.rs
//! indexes by position, so the length invariant is the thing that matters most.

mod common;

use rust_top::data::disk::DiskData;

fn collected() -> (DiskData, sysinfo::System) {
    let mut sys = common::loaded_system();
    let mut disks = DiskData::default();
    disks.refresh(&mut sys);
    disks.collect_all(&mut sys);
    (disks, sys)
}

fn lengths(d: &DiskData) -> [usize; 7] {
    [
        d.disk_name.len(),
        d.filesytem.len(),
        d.mount.len(),
        d.total.len(),
        d.available.len(),
        d.curr_read.len(),
        d.curr_write.len(),
    ]
}

#[test]
fn default_starts_completely_empty() {
    let d = DiskData::default();
    assert_eq!(lengths(&d), [0; 7]);
}

#[test]
fn all_parallel_vectors_have_the_same_length() {
    let (d, _sys) = collected();
    let l = lengths(&d);
    assert!(
        l.iter().all(|n| *n == l[0]),
        "disk_table.rs indexes all seven vectors by the same i; lengths were {l:?}"
    );
}

#[test]
fn collect_all_is_idempotent() {
    let mut sys = common::loaded_system();
    let mut d = DiskData::default();
    d.refresh(&mut sys);
    d.collect_all(&mut sys);
    let first = d.disk_name.clone();

    d.collect_all(&mut sys);
    assert_eq!(
        d.disk_name, first,
        "collect_all must overwrite, not append"
    );
    let l = lengths(&d);
    assert!(l.iter().all(|n| *n == l[0]));
}

#[test]
fn refresh_can_be_called_repeatedly_without_growing_state() {
    let mut sys = common::loaded_system();
    let mut d = DiskData::default();
    for _ in 0..3 {
        d.refresh(&mut sys);
        d.collect_all(&mut sys);
    }
    let l = lengths(&d);
    assert!(l.iter().all(|n| *n == l[0]));
}

#[test]
fn mount_points_are_sorted_ascending() {
    let (d, _sys) = collected();
    let mut sorted = d.mount.clone();
    sorted.sort();
    assert_eq!(d.mount, sorted, "collect_all sorts rows by mount point");
}

#[test]
fn available_space_never_exceeds_total() {
    let (d, _sys) = collected();
    for i in 0..d.total.len() {
        assert!(
            d.available[i] <= d.total[i],
            "{} reported {} available of {} total",
            d.mount[i],
            d.available[i],
            d.total[i]
        );
    }
}

#[test]
fn no_disk_metadata_field_is_blank() {
    let (d, _sys) = collected();
    for i in 0..d.disk_name.len() {
        assert!(!d.disk_name[i].is_empty(), "row {i} has a blank device name");
        assert!(!d.mount[i].is_empty(), "row {i} has a blank mount point");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn only_real_block_devices_are_kept() {
    let (d, _sys) = collected();
    for name in &d.disk_name {
        assert!(
            name.starts_with("/dev/"),
            "the linux filter should drop non-/dev entries, kept {name:?}"
        );
    }
}

#[test]
fn io_columns_repeat_a_single_system_wide_total() {
    // Current behaviour: get_total_io_read/write sum every process and the same
    // figure is cloned into every row. These are NOT per-disk numbers.
    let (d, _sys) = collected();
    if d.curr_read.len() > 1 {
        assert!(
            d.curr_read.windows(2).all(|w| w[0] == w[1]),
            "curr_read is a system-wide total repeated per row"
        );
        assert!(
            d.curr_write.windows(2).all(|w| w[0] == w[1]),
            "curr_write is a system-wide total repeated per row"
        );
    }
}
