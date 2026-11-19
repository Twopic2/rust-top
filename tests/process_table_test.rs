//! `draw::widgets::process_table` — sorting, filtering, selection and the
//! rendered list. This is the widest piece of pure logic in the app.

mod common;

use common::{pids, proc_row, proc_row_with_io};
use ratatui::layout::Rect;
use rust_top::draw::widgets::process_table::{ProcInfoPopup, ProcessTable, SearchState};

fn table_with(rows: Vec<rust_top::processes::processdata::CollectProcessData>) -> ProcessTable {
    let mut t = ProcessTable::new();
    t.proc_table = rows;
    t
}

// --------------------------------------------------------------- construction

#[test]
fn new_table_starts_empty_and_unfiltered() {
    let t = ProcessTable::new();
    assert_eq!(t.selected_pid, 0);
    assert!(t.proc_table.is_empty());
    assert!(t.filtered_table.is_empty());
    assert!(t.sorted_proc.is_empty());
    assert!(t.search_input.is_empty());
    assert!(t.search_state == SearchState::NoSearch);
    assert!(!t.is_searching());
    assert!(!t.is_filter_input_active());
}

// ------------------------------------------------------------------- sorting

#[test]
fn sorts_by_cpu_descending() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![
        proc_row(1, "low", "low", 10.0, 0.0),
        proc_row(2, "high", "high", 50.0, 0.0),
        proc_row(3, "mid", "mid", 30.0, 0.0),
    ]);
    assert_eq!(pids(&sorted), vec![2, 3, 1]);
}

#[test]
fn ties_break_on_ascending_pid() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![
        proc_row(10, "a", "a", 5.0, 0.0),
        proc_row(5, "b", "b", 5.0, 0.0),
        proc_row(7, "c", "c", 5.0, 0.0),
    ]);
    assert_eq!(pids(&sorted), vec![5, 7, 10]);
}

#[test]
fn nan_cpu_usage_sorts_as_zero_instead_of_panicking() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![
        proc_row(1, "nan", "nan", f32::NAN, 0.0),
        proc_row(2, "zero", "zero", 0.0, 0.0),
        proc_row(3, "busy", "busy", 1.0, 0.0),
    ]);
    // NaN is coerced to 0.0, so pid 1 and pid 2 tie and fall back to pid order.
    assert_eq!(pids(&sorted), vec![3, 1, 2]);
}

#[test]
fn infinite_cpu_usage_sorts_to_the_top() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![
        proc_row(1, "normal", "normal", 99.0, 0.0),
        proc_row(2, "inf", "inf", f32::INFINITY, 0.0),
    ]);
    assert_eq!(pids(&sorted), vec![2, 1]);
}

#[test]
fn sorting_an_empty_list_yields_an_empty_list() {
    let t = ProcessTable::new();
    assert!(t.get_sorted_processes(vec![]).is_empty());
}

#[test]
fn sorting_a_single_row_returns_it_unchanged() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![proc_row(9, "only", "only", 4.0, 1.0)]);
    assert_eq!(pids(&sorted), vec![9]);
    assert_eq!(sorted[0].program, "only");
}

#[test]
fn rows_sharing_a_pid_and_cpu_value_collapse_into_one() {
    // get_sorted_processes keys a BTreeMap on (cpu, pid), so identical keys
    // overwrite each other and the last row wins.
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![
        proc_row(9, "first", "first", 1.0, 0.0),
        proc_row(9, "second", "second", 1.0, 0.0),
    ]);
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0].program, "second");
}

#[test]
fn same_pid_with_different_cpu_values_is_kept_twice() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![
        proc_row(9, "a", "a", 1.0, 0.0),
        proc_row(9, "b", "b", 2.0, 0.0),
    ]);
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0].program, "b");
}

#[test]
fn sorting_preserves_every_field() {
    let t = ProcessTable::new();
    let sorted = t.get_sorted_processes(vec![proc_row_with_io(
        7,
        "prog",
        "prog --flag",
        3.5,
        2.5,
        111,
        222,
    )]);
    let row = &sorted[0];
    assert_eq!(row.pid, 7);
    assert_eq!(row.program, "prog");
    assert_eq!(row.command, "prog --flag");
    assert_eq!(row.cpu_usage_percent, 3.5);
    assert_eq!(row.mem_usage_percent, 2.5);
    let io = row.disk_io.as_ref().expect("io preserved");
    assert_eq!((io.read_bytes, io.write_bytes), (111, 222));
}

// ------------------------------------------------------------------ filtering

#[test]
fn filter_matches_a_program_name_prefix() {
    let mut t = table_with(vec![
        proc_row(1, "firefox", "firefox", 1.0, 0.0),
        proc_row(2, "bash", "bash", 1.0, 0.0),
    ]);
    t.search_input = "fire".into();
    t.apply_filter();
    assert_eq!(pids(&t.filtered_table), vec![1]);
}

#[test]
fn filter_matches_a_command_prefix_too() {
    let mut t = table_with(vec![
        proc_row(1, "unrelated", "firefox --new-window", 1.0, 0.0),
        proc_row(2, "bash", "bash", 1.0, 0.0),
    ]);
    t.search_input = "firefox".into();
    t.apply_filter();
    assert_eq!(pids(&t.filtered_table), vec![1]);
}

#[test]
fn filter_is_case_insensitive_on_both_sides() {
    let mut t = table_with(vec![proc_row(1, "FireFox", "FireFox", 1.0, 0.0)]);
    t.search_input = "FIRE".into();
    t.apply_filter();
    assert_eq!(pids(&t.filtered_table), vec![1]);
}

#[test]
fn filter_matches_prefixes_only_not_substrings() {
    let mut t = table_with(vec![proc_row(1, "firefox", "firefox", 1.0, 0.0)]);
    t.search_input = "fox".into();
    t.apply_filter();
    assert!(
        t.filtered_table.is_empty(),
        "starts_with must not match mid-word"
    );
}

#[test]
fn an_empty_query_matches_everything() {
    let mut t = table_with(vec![
        proc_row(1, "a", "a", 1.0, 0.0),
        proc_row(2, "b", "b", 1.0, 0.0),
    ]);
    t.search_input = String::new();
    t.apply_filter();
    assert_eq!(pids(&t.filtered_table), vec![1, 2]);
}

#[test]
fn a_query_matching_nothing_empties_the_filtered_table() {
    let mut t = table_with(vec![proc_row(1, "a", "a", 1.0, 0.0)]);
    t.search_input = "zzz".into();
    t.apply_filter();
    assert!(t.filtered_table.is_empty());
}

#[test]
fn apply_filter_preserves_the_source_table() {
    let mut t = table_with(vec![
        proc_row(1, "firefox", "firefox", 1.0, 0.0),
        proc_row(2, "bash", "bash", 1.0, 0.0),
    ]);
    t.search_input = "fire".into();
    t.apply_filter();

    assert_eq!(pids(&t.filtered_table), vec![1]);
    assert_eq!(pids(&t.proc_table), vec![1, 2]);
}

#[test]
fn filtering_twice_without_a_refresh_keeps_matching_rows() {
    let mut t = table_with(vec![proc_row(1, "firefox", "firefox", 1.0, 0.0)]);
    t.search_input = "fire".into();
    t.apply_filter();
    t.search_input = "f".into();
    t.apply_filter();

    assert_eq!(pids(&t.filtered_table), vec![1]);
}

// ------------------------------------------------------------- search states

#[test]
fn is_searching_covers_both_active_search_states() {
    let mut t = ProcessTable::new();

    t.search_state = SearchState::NoSearch;
    assert!(!t.is_searching());

    t.search_state = SearchState::Searching;
    assert!(t.is_searching());

    t.search_state = SearchState::FilterApplied;
    assert!(t.is_searching());
}

#[test]
fn only_the_typing_state_activates_the_filter_input() {
    let mut t = ProcessTable::new();

    t.search_state = SearchState::Searching;
    assert!(t.is_filter_input_active());

    t.search_state = SearchState::FilterApplied;
    assert!(!t.is_filter_input_active());

    t.search_state = SearchState::NoSearch;
    assert!(!t.is_filter_input_active());
}

// ------------------------------------------------------------------- deletion

#[test]
fn deleting_an_entry_removes_it_from_all_three_tables() {
    let mut t = ProcessTable::new();
    t.proc_table = vec![
        proc_row(1, "a", "a", 1.0, 0.0),
        proc_row(2, "b", "b", 1.0, 0.0),
    ];
    t.filtered_table = t.proc_table.clone();
    t.sorted_proc = t.proc_table.clone();

    t.delete_table_entry(1);

    assert_eq!(pids(&t.proc_table), vec![2]);
    assert_eq!(pids(&t.filtered_table), vec![2]);
    assert_eq!(pids(&t.sorted_proc), vec![2]);
}

#[test]
fn deleting_clears_the_selection() {
    let mut t = ProcessTable::new();
    t.proc_table = vec![proc_row(1, "a", "a", 1.0, 0.0)];
    t.selected_pid = 1;
    t.delete_table_entry(1);
    assert_eq!(t.selected_pid, 0);
}

#[test]
fn deleting_an_unknown_pid_is_a_no_op_on_the_rows() {
    let mut t = ProcessTable::new();
    t.proc_table = vec![proc_row(1, "a", "a", 1.0, 0.0)];
    t.delete_table_entry(999);
    assert_eq!(pids(&t.proc_table), vec![1]);
}

#[test]
fn deleting_removes_every_row_sharing_that_pid() {
    let mut t = ProcessTable::new();
    t.proc_table = vec![
        proc_row(1, "a", "a", 1.0, 0.0),
        proc_row(1, "b", "b", 2.0, 0.0),
    ];
    t.delete_table_entry(1);
    assert!(t.proc_table.is_empty());
}

// ------------------------------------------------------------------ rendering

#[test]
fn render_draws_the_column_header() {
    let mut t = table_with(vec![proc_row(1, "bash", "bash", 1.0, 1.0)]);
    let buf = common::render(80, 6, |f| f.render_widget(&mut t, Rect::new(0, 0, 80, 6)));
    let text = common::buffer_text(&buf);

    for header in ["PID", "CPU", "MEM", "Program", "Command"] {
        assert!(text.contains(header), "missing {header} in:\n{text}");
    }
}

#[test]
fn render_lists_rows_sorted_by_cpu_descending() {
    let mut t = table_with(vec![
        proc_row(111, "low", "low", 1.0, 0.0),
        proc_row(222, "high", "high", 90.0, 0.0),
    ]);
    let buf = common::render(80, 6, |f| f.render_widget(&mut t, Rect::new(0, 0, 80, 6)));
    let text = common::buffer_text(&buf);

    let high = text.find("222").expect("high-cpu row rendered");
    let low = text.find("111").expect("low-cpu row rendered");
    assert!(high < low, "busiest process should render first:\n{text}");
}

#[test]
fn render_populates_sorted_proc_as_a_side_effect() {
    let mut t = table_with(vec![
        proc_row(1, "a", "a", 1.0, 0.0),
        proc_row(2, "b", "b", 9.0, 0.0),
    ]);
    assert!(t.sorted_proc.is_empty());
    let _ = common::render(80, 6, |f| f.render_widget(&mut t, Rect::new(0, 0, 80, 6)));
    assert_eq!(pids(&t.sorted_proc), vec![2, 1]);
}

#[test]
fn render_truncates_the_command_at_the_first_space() {
    let mut t = table_with(vec![proc_row(
        1,
        "firefox",
        "firefox --new-window --private",
        5.0,
        1.0,
    )]);
    let buf = common::render(90, 5, |f| f.render_widget(&mut t, Rect::new(0, 0, 90, 5)));
    let text = common::buffer_text(&buf);

    assert!(text.contains("firefox"));
    assert!(
        !text.contains("--new-window"),
        "arguments should be trimmed:\n{text}"
    );
}

#[test]
fn render_shows_the_filter_bar_only_while_typing() {
    let mut t = table_with(vec![proc_row(1, "bash", "bash", 1.0, 0.0)]);

    let without = common::buffer_text(&common::render(60, 8, |f| {
        f.render_widget(&mut t, Rect::new(0, 0, 60, 8))
    }));
    assert!(!without.contains("Filter"));

    t.search_state = SearchState::Searching;
    t.search_input = "fire".into();
    let with = common::buffer_text(&common::render(60, 8, |f| {
        f.render_widget(&mut t, Rect::new(0, 0, 60, 8))
    }));
    assert!(with.contains("Filter"), "filter bar missing:\n{with}");
    assert!(with.contains("/fire"), "query not echoed:\n{with}");
}

#[test]
fn render_uses_the_filtered_rows_once_a_filter_is_applied() {
    let mut t = table_with(vec![
        proc_row(111, "keep", "keep", 1.0, 0.0),
        proc_row(222, "drop", "drop", 1.0, 0.0),
    ]);
    t.search_state = SearchState::FilterApplied;
    t.filtered_table = vec![proc_row(111, "keep", "keep", 1.0, 0.0)];

    let text = common::buffer_text(&common::render(60, 8, |f| {
        f.render_widget(&mut t, Rect::new(0, 0, 60, 8))
    }));
    assert!(text.contains("keep"));
    assert!(
        !text.contains("222"),
        "unfiltered row leaked into the list:\n{text}"
    );
}

#[test]
fn rendering_an_empty_table_still_draws_the_header() {
    let mut t = ProcessTable::new();
    let text = common::buffer_text(&common::render(60, 5, |f| {
        f.render_widget(&mut t, Rect::new(0, 0, 60, 5))
    }));
    assert!(text.contains("PID"));
}

#[test]
fn rendering_into_a_tiny_area_does_not_panic() {
    let mut t = table_with(vec![proc_row(1, "bash", "bash", 1.0, 0.0)]);
    let _ = common::render(4, 2, |f| f.render_widget(&mut t, Rect::new(0, 0, 4, 2)));
    let _ = common::render(1, 1, |f| f.render_widget(&mut t, Rect::new(0, 0, 1, 1)));
}

// --------------------------------------------------------------- click target

#[test]
fn clicking_before_the_first_render_selects_nothing() {
    let mut t = table_with(vec![proc_row(42, "a", "a", 1.0, 0.0)]);
    t.handle_click(5, 3);
    assert_eq!(t.selected_pid, 0, "render_area is still zero-sized");
}

#[test]
fn clicking_outside_the_rendered_area_is_ignored() {
    let mut t = table_with(vec![proc_row(42, "a", "a", 1.0, 0.0)]);
    let _ = common::render(60, 8, |f| f.render_widget(&mut t, Rect::new(0, 0, 60, 8)));

    t.handle_click(200, 3);
    assert_eq!(t.selected_pid, 0, "column outside the widget");
    t.handle_click(5, 100);
    assert_eq!(t.selected_pid, 0, "row outside the widget");
}

#[test]
fn clicking_selects_a_process() {
    let mut t = table_with(vec![
        proc_row(111, "busy", "busy", 90.0, 0.0),
        proc_row(222, "idle", "idle", 1.0, 0.0),
    ]);
    let _ = common::render(60, 8, |f| f.render_widget(&mut t, Rect::new(0, 0, 60, 8)));

    t.handle_click(5, 1);
    assert_eq!(
        t.selected_pid, 111,
        "first sorted row is the busiest process"
    );
}

#[test]
fn clicking_selects_the_process_on_the_rendered_row() {
    let mut t = table_with(vec![
        proc_row(111, "a", "a", 90.0, 0.0),
        proc_row(222, "b", "b", 50.0, 0.0),
        proc_row(333, "c", "c", 10.0, 0.0),
    ]);
    let _ = common::render(60, 8, |f| f.render_widget(&mut t, Rect::new(0, 0, 60, 8)));
    assert_eq!(pids(&t.sorted_proc), vec![111, 222, 333]);

    t.handle_click(5, 0);
    assert_eq!(t.selected_pid, 0, "the header is not a process");
    for (row, pid) in [(1, 111), (2, 222), (3, 333)] {
        t.handle_click(5, row);
        assert_eq!(t.selected_pid, pid, "row {row}");
    }

    t.handle_click(5, 4);
    assert_eq!(t.selected_pid, 333);
}

#[test]
fn clicking_past_the_last_row_keeps_the_previous_selection() {
    let mut t = table_with(vec![proc_row(111, "a", "a", 1.0, 0.0)]);
    let _ = common::render(60, 8, |f| f.render_widget(&mut t, Rect::new(0, 0, 60, 8)));

    t.handle_click(5, 1);
    assert_eq!(t.selected_pid, 111);
    t.handle_click(5, 7); // below the single data row
    assert_eq!(
        t.selected_pid, 111,
        "an out-of-range index must not clear the selection"
    );
}

#[test]
fn clicking_respects_a_non_zero_widget_origin() {
    let mut t = table_with(vec![proc_row(111, "a", "a", 1.0, 0.0)]);
    let area = Rect::new(10, 4, 40, 6);
    let _ = common::render(60, 12, |f| f.render_widget(&mut t, area));

    t.handle_click(5, 5);
    assert_eq!(t.selected_pid, 0, "column left of the widget");

    t.handle_click(15, 5);
    assert_eq!(t.selected_pid, 111);
}

// ------------------------------------------------------------ ProcInfoPopup

#[test]
fn clicking_while_typing_ignores_the_filter_bar_and_header() {
    let mut t = table_with(vec![proc_row(111, "bash", "bash", 1.0, 0.0)]);
    t.search_state = SearchState::Searching;
    t.apply_filter();
    let area = Rect::new(10, 2, 40, 8);
    let _ = common::render(60, 12, |f| f.render_widget(&mut t, area));

    for row in 2..6 {
        t.handle_click(15, row);
        assert_eq!(t.selected_pid, 0, "filter/header row {row}");
    }
    t.handle_click(15, 6);
    assert_eq!(t.selected_pid, 111);
}

#[test]
fn clicking_a_scrolled_list_selects_the_visible_process() {
    let mut t = table_with(
        (1..=10)
            .map(|pid| proc_row(pid, "prog", "prog", 1.0, 0.0))
            .collect(),
    );
    t.selected_pid = 10;
    let buffer = common::render(60, 3, |f| f.render_widget(&mut t, Rect::new(0, 0, 60, 3)));
    let text = common::buffer_text(&buffer);
    for (row, line) in text.lines().enumerate() {
        let visible_pid: u32 = line.split_whitespace().next().unwrap().parse().unwrap();
        t.handle_click(5, row as u16);
        assert_eq!(t.selected_pid, visible_pid);
    }
}

#[test]
fn info_popup_starts_hidden() {
    let p = ProcInfoPopup::new();
    assert!(!p.visable);
    assert_eq!(p.selected_pid, 0);
    assert!(p.total_proc.proc_table.is_empty());
}

#[test]
fn hidden_info_popup_draws_nothing() {
    let mut p = ProcInfoPopup::new();
    p.total_proc.proc_table = vec![proc_row(42, "prog", "prog", 1.0, 1.0)];
    p.selected_pid = 42;

    let buf = common::render(60, 20, |f| f.render_widget(&p, Rect::new(0, 0, 60, 20)));
    assert!(
        common::buffer_is_blank(&buf),
        "hidden popup rendered content"
    );
}

#[test]
fn visible_info_popup_shows_the_selected_process() {
    let mut p = ProcInfoPopup::new();
    p.visable = true;
    p.selected_pid = 42;
    p.total_proc.proc_table = vec![proc_row(42, "myprog", "myprog --x", 12.5, 3.5)];

    let text = common::buffer_text(&common::render(80, 24, |f| {
        f.render_widget(&p, Rect::new(0, 0, 80, 24))
    }));
    assert!(text.contains("Process Info"), "missing title:\n{text}");
    assert!(text.contains("42"), "missing pid:\n{text}");
    assert!(text.contains("myprog"), "missing program:\n{text}");
    assert!(text.contains("Base User"), "missing user:\n{text}");
}

#[test]
fn info_popup_shows_io_rows_only_when_io_was_collected() {
    let mut with_io = ProcInfoPopup::new();
    with_io.visable = true;
    with_io.selected_pid = 42;
    with_io.total_proc.proc_table =
        vec![proc_row_with_io(42, "prog", "prog", 1.0, 1.0, 4096, 8192)];
    let text = common::buffer_text(&common::render(80, 24, |f| {
        f.render_widget(&with_io, Rect::new(0, 0, 80, 24))
    }));
    assert!(text.contains("Read IO"), "io rows missing:\n{text}");
    assert!(text.contains("4096"));
    assert!(text.contains("8192"));

    let mut without_io = ProcInfoPopup::new();
    without_io.visable = true;
    without_io.selected_pid = 42;
    without_io.total_proc.proc_table = vec![proc_row(42, "prog", "prog", 1.0, 1.0)];
    let text = common::buffer_text(&common::render(80, 24, |f| {
        f.render_widget(&without_io, Rect::new(0, 0, 80, 24))
    }));
    assert!(
        !text.contains("Read IO"),
        "io rows should be omitted:\n{text}"
    );
}

#[test]
fn info_popup_for_an_unknown_pid_still_draws_its_frame() {
    let mut p = ProcInfoPopup::new();
    p.visable = true;
    p.selected_pid = 999;
    p.total_proc.proc_table = vec![proc_row(42, "prog", "prog", 1.0, 1.0)];

    let text = common::buffer_text(&common::render(80, 24, |f| {
        f.render_widget(&p, Rect::new(0, 0, 80, 24))
    }));
    assert!(text.contains("Process Info"));
    assert!(
        !text.contains("Program:"),
        "no detail rows for a missing pid:\n{text}"
    );
}
