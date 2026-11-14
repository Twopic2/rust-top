//! The remaining draw widgets: core grid, CPU usage bar, disk table,
//! temperature panel, network graph and the about popup.

mod common;

use ratatui::layout::Rect;
use rust_top::data::disk::DiskData;
use rust_top::draw::widgets::about_popup::AboutPopUp;
use rust_top::draw::widgets::cpu_bar::{BarColorScheme, TotalCoreBar};
use rust_top::draw::widgets::cpu_graph::{ColorScheme, MultiCoreGraph};
use rust_top::draw::widgets::disk_table::DiskTable;
use rust_top::draw::widgets::network_graph::NetworkGraph;
use rust_top::draw::widgets::temp_widget::TempWidget;
use sysinfo::System;

// ------------------------------------------------------------ MultiCoreGraph

#[test]
fn core_graph_starts_with_one_zeroed_entry_per_core() {
    let g = MultiCoreGraph::new(4, ColorScheme::Cyan);
    assert_eq!(g.cores.len(), 4);
    for i in 0..4 {
        assert_eq!(g.cores.get(&i), Some(&0.0), "core {i}");
    }
}

#[test]
fn core_graph_preserves_core_ordering() {
    let g = MultiCoreGraph::new(8, ColorScheme::Green);
    let keys: Vec<usize> = g.cores.keys().copied().collect();
    assert_eq!(keys, (0..8).collect::<Vec<_>>(), "IndexMap must keep insertion order");
}

#[test]
fn core_graph_renders_its_title_and_a_frequency_label() {
    let mut g = MultiCoreGraph::new(2, ColorScheme::Cyan);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(50, 8, |f| {
        g.render(f, Rect::new(0, 0, 50, 8), &mut sys)
    }));

    assert!(text.contains("Cpu Cores"), "missing title:\n{text}");
    assert!(text.contains("GHz"), "missing frequency label:\n{text}");
}

#[test]
fn core_graph_renders_one_labelled_bar_per_core() {
    let mut g = MultiCoreGraph::new(3, ColorScheme::Cyan);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(50, 10, |f| {
        g.render(f, Rect::new(0, 0, 50, 10), &mut sys)
    }));

    for label in ["C0", "C1", "C2"] {
        assert!(text.contains(label), "missing {label}:\n{text}");
    }
}

#[test]
fn core_graph_renders_the_usage_written_into_cores() {
    let mut g = MultiCoreGraph::new(1, ColorScheme::Cyan);
    g.cores.insert(0, 55.0);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(50, 6, |f| {
        g.render(f, Rect::new(0, 0, 50, 6), &mut sys)
    }));

    assert!(text.contains("55%"), "usage not rendered:\n{text}");
}

#[test]
fn core_graph_draws_a_filled_and_an_empty_segment() {
    let mut g = MultiCoreGraph::new(1, ColorScheme::Cyan);
    g.cores.insert(0, 50.0);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(50, 6, |f| {
        g.render(f, Rect::new(0, 0, 50, 6), &mut sys)
    }));

    assert!(text.contains('█'), "no filled segment:\n{text}");
    assert!(text.contains('░'), "no empty segment:\n{text}");
}

#[test]
fn core_graph_at_zero_percent_draws_no_filled_segment() {
    let mut g = MultiCoreGraph::new(1, ColorScheme::Cyan);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(50, 6, |f| {
        g.render(f, Rect::new(0, 0, 50, 6), &mut sys)
    }));
    assert!(!text.contains('█'), "0% should be entirely empty:\n{text}");
}

#[test]
fn core_graph_with_no_cores_renders_only_the_frame() {
    let mut g = MultiCoreGraph::new(0, ColorScheme::Red);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(40, 6, |f| {
        g.render(f, Rect::new(0, 0, 40, 6), &mut sys)
    }));
    assert!(text.contains("Cpu Cores"));
    assert!(!text.contains("C0"));
}

#[test]
fn core_graph_survives_a_tiny_area() {
    let mut g = MultiCoreGraph::new(16, ColorScheme::Magenta);
    let mut sys = System::new();
    for (w, h) in [(1, 1), (3, 2), (4, 3), (12, 4)] {
        let _ = common::render(20, 10, |f| g.render(f, Rect::new(0, 0, w, h), &mut sys));
    }
}

#[test]
fn core_graph_clips_to_the_available_height() {
    // 32 cores at one per row cannot fit in a 6-row box; it must clip, not panic.
    let mut g = MultiCoreGraph::new(32, ColorScheme::Blue);
    let mut sys = System::new();
    let text = common::buffer_text(&common::render(40, 6, |f| {
        g.render(f, Rect::new(0, 0, 40, 6), &mut sys)
    }));
    assert!(!text.contains("C31"), "the last core should be clipped away");
}

#[test]
fn every_colour_scheme_renders() {
    let mut sys = System::new();
    for scheme in [
        ColorScheme::Green,
        ColorScheme::Cyan,
        ColorScheme::Red,
        ColorScheme::Yellow,
        ColorScheme::Blue,
        ColorScheme::Magenta,
    ] {
        let mut g = MultiCoreGraph::new(2, scheme);
        g.cores.insert(0, 95.0); // exercise the top of every gradient
        g.cores.insert(1, 10.0);
        let _ = common::render(50, 8, |f| g.render(f, Rect::new(0, 0, 50, 8), &mut sys));
    }
}

// -------------------------------------------------------------- TotalCoreBar

#[test]
fn total_bar_averages_the_core_usages() {
    let mut bar = TotalCoreBar::new(BarColorScheme::Green);
    bar.update(&[0.0, 50.0, 100.0]);
    let text = common::buffer_text(&common::render(60, 3, |f| bar.render(f, Rect::new(0, 0, 60, 3))));
    assert!(text.contains("50.0%"), "expected the mean of 0/50/100:\n{text}");
}

#[test]
fn total_bar_handles_an_empty_core_list() {
    let mut bar = TotalCoreBar::new(BarColorScheme::Green);
    bar.update(&[]);
    let text = common::buffer_text(&common::render(60, 3, |f| bar.render(f, Rect::new(0, 0, 60, 3))));
    assert!(text.contains("0.0%"), "empty input must not divide by zero:\n{text}");
}

#[test]
fn total_bar_recomputes_on_every_update() {
    let mut bar = TotalCoreBar::new(BarColorScheme::Green);
    bar.update(&[100.0, 100.0]);
    bar.update(&[10.0, 20.0]);
    let text = common::buffer_text(&common::render(60, 3, |f| bar.render(f, Rect::new(0, 0, 60, 3))));
    assert!(text.contains("15.0%"), "stale average:\n{text}");
    assert!(!text.contains("100.0%"));
}

#[test]
fn total_bar_renders_its_title_and_both_segments() {
    let mut bar = TotalCoreBar::new(BarColorScheme::Green);
    bar.update(&[40.0]);
    let text = common::buffer_text(&common::render(60, 3, |f| bar.render(f, Rect::new(0, 0, 60, 3))));

    assert!(text.contains("Total CPU Usage"));
    assert!(text.contains('█'), "no filled segment:\n{text}");
    assert!(text.contains('░'), "no empty segment:\n{text}");
}

#[test]
fn total_bar_at_full_load_has_no_empty_segment() {
    let mut bar = TotalCoreBar::new(BarColorScheme::Red);
    bar.update(&[100.0]);
    let text = common::buffer_text(&common::render(60, 3, |f| bar.render(f, Rect::new(0, 0, 60, 3))));
    assert!(text.contains("100.0%"));
    assert!(!text.contains('░'), "a full bar should have no empty segment:\n{text}");
}

#[test]
fn total_bar_survives_a_narrow_area() {
    let mut bar = TotalCoreBar::new(BarColorScheme::Yellow);
    bar.update(&[50.0]);
    for w in [1, 4, 8, 11] {
        let _ = common::render(20, 3, |f| bar.render(f, Rect::new(0, 0, w, 3)));
    }
}

// ----------------------------------------------------------------- DiskTable

fn synthetic_disks(n: usize) -> DiskData {
    let mut d = DiskData::default();
    for i in 0..n {
        d.disk_name.push(format!("/dev/sda{i}"));
        d.filesytem.push("ext4".into());
        d.mount.push(format!("/mnt/{i}"));
        d.total.push(512 * 1024 * 1024 * 1024);
        d.available.push(256 * 1024 * 1024 * 1024);
        d.curr_read.push(1024 * 1024);
        d.curr_write.push(2 * 1024 * 1024);
    }
    d
}

#[test]
fn disk_table_height_has_a_floor_of_four_rows() {
    let table = DiskTable::new();
    assert_eq!(table.get_height(&synthetic_disks(0)), 4);
    assert_eq!(table.get_height(&synthetic_disks(1)), 4);
}

#[test]
fn disk_table_height_grows_with_the_row_count() {
    let table = DiskTable::new();
    assert_eq!(table.get_height(&synthetic_disks(2)), 5);
    assert_eq!(table.get_height(&synthetic_disks(5)), 8);
    assert_eq!(table.get_height(&synthetic_disks(20)), 23);
}

#[test]
fn disk_table_renders_its_headers() {
    let mut table = DiskTable::new();
    let disks = synthetic_disks(1);
    let text = common::buffer_text(&common::render(90, 8, |f| {
        table.render(f, Rect::new(0, 0, 90, 8), &disks)
    }));

    assert!(text.contains("Disk Usage"), "missing title:\n{text}");
    assert!(text.contains("Kernel"), "missing kernel label:\n{text}");
    for header in ["FS", "Mount", "Total", "Available"] {
        assert!(text.contains(header), "missing {header} column:\n{text}");
    }
}

#[test]
fn disk_table_renders_one_row_per_disk_with_formatted_sizes() {
    let mut table = DiskTable::new();
    let disks = synthetic_disks(2);
    let text = common::buffer_text(&common::render(110, 8, |f| {
        table.render(f, Rect::new(0, 0, 110, 8), &disks)
    }));

    assert!(text.contains("sda0"), "first disk missing:\n{text}");
    assert!(text.contains("sda1"), "second disk missing:\n{text}");
    assert!(text.contains("512.00 GB"), "total not formatted:\n{text}");
    assert!(text.contains("256.00 GB"), "available not formatted:\n{text}");
}

#[test]
fn disk_table_renders_an_empty_disk_list() {
    let mut table = DiskTable::new();
    let disks = synthetic_disks(0);
    let text = common::buffer_text(&common::render(90, 6, |f| {
        table.render(f, Rect::new(0, 0, 90, 6), &disks)
    }));
    assert!(text.contains("Disk Usage"));
}

#[test]
fn disk_table_survives_a_tiny_area() {
    let mut table = DiskTable::new();
    let disks = synthetic_disks(3);
    for (w, h) in [(1, 1), (3, 3), (4, 2), (10, 4)] {
        let _ = common::render(20, 8, |f| table.render(f, Rect::new(0, 0, w, h), &disks));
    }
}

// ---------------------------------------------------------------- TempWidget

#[test]
fn temp_widget_defaults_to_the_minimum_geometry() {
    let w = TempWidget::default();
    assert_eq!(w.get_height(), 3, "one row plus borders");
    assert_eq!(w.get_length(), 20, "the 20-column floor");
}

#[test]
fn temp_widget_height_tracks_the_sensors_it_matched() {
    let mut w = TempWidget::default();
    w.filter();
    let h = w.get_height();
    assert!(h >= 3, "height {h} dropped below the minimum");
    assert!(h <= 6, "at most four sensors are matched, got height {h}");
}

#[test]
fn temp_widget_length_never_drops_below_the_floor() {
    let mut w = TempWidget::default();
    w.filter();
    assert!(w.get_length() >= 20);
}

#[test]
fn temp_widget_filter_is_idempotent_on_geometry_bounds() {
    let mut w = TempWidget::default();
    w.filter();
    let once = w.get_height();
    w.filter();
    let twice = w.get_height();
    assert_eq!(twice, once, "filtering the same sensors must preserve the height");
}

#[test]
fn temp_widget_renders_its_panel_when_sensors_exist() {
    use rust_top::data::temp::TempData;
    let mut w = TempWidget::default();
    w.filter();
    let text = common::buffer_text(&common::render(40, 8, |f| w.render(f, Rect::new(0, 0, 40, 8))));

    if TempData::all_temps().is_some() {
        assert!(text.contains("Temperatures"), "missing title:\n{text}");
    }
}

#[test]
fn temp_widget_survives_a_tiny_area() {
    let mut w = TempWidget::default();
    w.filter();
    for (width, height) in [(1, 1), (3, 2), (8, 3)] {
        let _ = common::render(20, 8, |f| w.render(f, Rect::new(0, 0, width, height)));
    }
}

// --------------------------------------------------------------- NetworkGraph

#[test]
fn network_graph_renders_rx_and_tx_series() {
    let mut g = NetworkGraph::new(60);
    g.update();
    let text = common::buffer_text(&common::render(110, 24, |f| g.render(f, Rect::new(0, 0, 110, 24))));

    assert!(text.contains("RX:"), "missing rx series:\n{text}");
    assert!(text.contains("TX:"), "missing tx series:\n{text}");
    assert!(text.contains("All:"), "missing cumulative totals:\n{text}");
}

#[test]
fn network_graph_renders_before_any_update() {
    let mut g = NetworkGraph::new(60);
    let text = common::buffer_text(&common::render(110, 24, |f| g.render(f, Rect::new(0, 0, 110, 24))));
    assert!(text.contains("RX:"), "an empty history must still render:\n{text}");
}

#[test]
fn network_graph_legend_needs_a_wide_pane() {
    // ratatui hides the legend when it would eat too much of the plot, so the
    // RX/TX byte counters vanish below roughly 100 columns.
    let mut g = NetworkGraph::new(60);
    g.update();

    let narrow = common::buffer_text(&common::render(90, 20, |f| g.render(f, Rect::new(0, 0, 90, 20))));
    assert!(!narrow.contains("RX:"), "legend unexpectedly shown at 90 cols:\n{narrow}");

    let wide = common::buffer_text(&common::render(100, 20, |f| g.render(f, Rect::new(0, 0, 100, 20))));
    assert!(wide.contains("RX:"), "legend missing at 100 cols:\n{wide}");
}

#[test]
fn network_graph_always_draws_its_frame_and_axes() {
    let mut g = NetworkGraph::new(60);
    g.update();
    let text = common::buffer_text(&common::render(80, 16, |f| g.render(f, Rect::new(0, 0, 80, 16))));
    assert!(text.contains("Local IP address:") || text.contains("Unable to get local IP"));
    assert!(text.contains("1.0K"), "y axis top label missing:\n{text}");
    assert!(text.contains("0s"), "x axis missing:\n{text}");
}

#[test]
fn network_graph_labels_the_time_axis_from_max_points() {
    let mut g = NetworkGraph::new(30);
    g.update();
    let text = common::buffer_text(&common::render(80, 16, |f| g.render(f, Rect::new(0, 0, 80, 16))));
    assert!(text.contains("60s"), "x axis should span max_points * 2 seconds:\n{text}");
    assert!(text.contains("0s"));
}

#[test]
fn network_graph_history_is_bounded_by_max_points() {
    // Push well past the cap; the ring buffer must not grow without limit or panic.
    let mut g = NetworkGraph::new(4);
    for _ in 0..20 {
        g.update();
    }
    let _ = common::render(80, 16, |f| g.render(f, Rect::new(0, 0, 80, 16)));
}

#[test]
fn network_graph_survives_a_tiny_area() {
    let mut g = NetworkGraph::new(10);
    g.update();
    for (w, h) in [(1, 1), (5, 3), (10, 4)] {
        let _ = common::render(20, 16, |f| g.render(f, Rect::new(0, 0, w, h)));
    }
}

// ----------------------------------------------------------------- AboutPopUp

#[test]
fn about_popup_starts_hidden() {
    assert!(!AboutPopUp::default().visable);
}

#[test]
fn hidden_about_popup_draws_nothing() {
    let popup = AboutPopUp::default();
    let buf = common::render(60, 20, |f| popup.render(f, Rect::new(0, 0, 60, 20)));
    assert!(common::buffer_is_blank(&buf), "hidden popup rendered content");
}

#[test]
fn visible_about_popup_shows_its_title_and_blurb() {
    let popup = AboutPopUp { visable: true };
    let text = common::buffer_text(&common::render(90, 24, |f| popup.render(f, Rect::new(0, 0, 90, 24))));

    assert!(text.contains("About Rust-top"), "missing title:\n{text}");
    assert!(text.contains("Terminal system monitoring"), "missing blurb:\n{text}");
}

#[test]
fn about_popup_lists_every_keybind_when_it_has_room() {
    let popup = AboutPopUp { visable: true };
    let text = common::buffer_text(&common::render(90, 60, |f| popup.render(f, Rect::new(0, 0, 90, 60))));

    assert!(text.contains("Basic Keybinds"), "missing keybind section:\n{text}");
    for binding in ["Quit", "Kill selected process", "Search/filter processes"] {
        assert!(text.contains(binding), "missing {binding}:\n{text}");
    }
}

#[test]
fn about_popup_shows_all_keybinds_in_a_standard_terminal() {
    let popup = AboutPopUp { visable: true };

    let short = common::buffer_text(&common::render(90, 24, |f| popup.render(f, Rect::new(0, 0, 90, 24))));
    assert!(short.contains("About Rust-top"), "title should still show:\n{short}");
    for binding in ["Basic Keybinds", "Quit", "Kill selected process", "Search/filter processes",
        "Increase refresh rate", "Decrease refresh rate"] {
        assert!(short.contains(binding), "missing {binding}:\n{short}");
    }

    let tall = common::buffer_text(&common::render(90, 40, |f| popup.render(f, Rect::new(0, 0, 90, 40))));
    assert!(tall.contains("Basic Keybinds"), "keybinds should fit at 40 rows:\n{tall}");
}

#[test]
fn visible_about_popup_dims_the_background() {
    let popup = AboutPopUp { visable: true };
    let text = common::buffer_text(&common::render(90, 24, |f| popup.render(f, Rect::new(0, 0, 90, 24))));
    assert!(text.contains('░'), "the blur layer should fill the backdrop:\n{text}");
}

#[test]
fn about_popup_survives_a_tiny_area() {
    let popup = AboutPopUp { visable: true };
    for (w, h) in [(1, 1), (4, 3), (10, 5)] {
        let _ = common::render(20, 10, |f| popup.render(f, Rect::new(0, 0, w, h)));
    }
}
