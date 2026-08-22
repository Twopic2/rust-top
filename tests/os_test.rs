//! `data::os` — kernel and hostname lookups. Both are unwrapped by the draw
//! layer, so a `None` here would panic the real app.

use rust_top::data::os::OsInfo;

#[test]
fn display_kernel_is_available() {
    let kernel = OsInfo::display_kernel();
    assert!(
        kernel.is_some(),
        "disk_table.rs unwraps this value; None would panic the app"
    );
    assert!(!kernel.unwrap().trim().is_empty());
}

#[test]
fn display_host_name_is_available() {
    let host = OsInfo::display_host_name();
    assert!(
        host.is_some(),
        "app.rs unwraps this value; None would panic the app"
    );
    assert!(!host.unwrap().trim().is_empty());
}

#[test]
fn lookups_are_stable_across_calls() {
    assert_eq!(OsInfo::display_kernel(), OsInfo::display_kernel());
    assert_eq!(OsInfo::display_host_name(), OsInfo::display_host_name());
}
