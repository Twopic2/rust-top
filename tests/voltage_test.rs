//! Voltage snapshots and rendering. Sensor initialization is serialized because
//! lm-sensors permits only one active instance per process; no voltage hardware
//! is required for the rendering fixtures.

mod common;

use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use rust_top::data::voltage::Voltage;
use rust_top::draw::widgets::voltage_bar::VoltageUsage;
use std::sync::{Mutex, MutexGuard};

static SENSOR_LOCK: Mutex<()> = Mutex::new(());

fn backend() -> (MutexGuard<'static, ()>, Voltage) {
    let guard = SENSOR_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let sensors = lm_sensors::Initializer::default()
        .config_path("/dev/null")
        .initialize()
        .expect("initialize lm-sensors with an empty configuration");
    (guard, Voltage::new(sensors))
}

fn widget(readings: &[(&str, f64)]) -> (MutexGuard<'static, ()>, VoltageUsage) {
    let (guard, mut voltage) = backend();
    voltage.voltage_read_points = readings
        .iter()
        .map(|(label, value)| (label.to_string(), *value))
        .collect();
    (guard, VoltageUsage::new(voltage))
}

#[test]
fn sensor_backend_refresh_replaces_readings_instead_of_appending() {
    let (_guard, mut voltage) = backend();
    assert!(voltage.voltage_read_points.is_empty());

    for _ in 0..3 {
        voltage
            .voltage_read_points
            .push(("stale-test-reading".into(), 123.0));
        voltage.refresh();
        assert!(voltage
            .voltage_read_points
            .iter()
            .all(|(label, _)| label != "stale-test-reading"));

        let count = voltage.voltage_read_points.len();
        voltage.refresh();
        assert_eq!(
            voltage.voltage_read_points.len(),
            count,
            "refresh must replace the snapshot"
        );
    }
}

#[test]
fn empty_voltage_widget_draws_its_title_and_border() {
    let (_guard, voltage) = widget(&[]);
    let text = common::buffer_text(&common::render(30, 5, |frame| {
        frame.render_widget(&voltage, frame.area());
    }));

    assert!(text.contains("Voltage"), "missing title:\n{text}");
    assert!(
        text.contains('┌') && text.contains('┘'),
        "missing border:\n{text}"
    );
    assert!(!text.contains(":"), "no readings should be shown:\n{text}");
}

#[test]
fn voltage_widget_renders_labels_values_and_units_in_order() {
    let (_guard, voltage) = widget(&[("Vcore", 1.25), ("in1", 0.0), ("negative rail", -12.0)]);
    let text = common::buffer_text(&common::render(35, 6, |frame| {
        frame.render_widget(&voltage, frame.area());
    }));

    let rows: Vec<_> = text.lines().collect();
    assert!(
        rows[1].contains("Vcore: 1.25V"),
        "first reading missing:\n{text}"
    );
    assert!(rows[2].contains("in1: 0V"), "zero reading missing:\n{text}");
    assert!(
        rows[3].contains("negative rail: -12V"),
        "negative reading missing:\n{text}"
    );
}

#[test]
fn repeated_rendering_preserves_the_voltage_snapshot() {
    let (_guard, voltage) = widget(&[("Vcore", 1.25), ("in1", 3.3)]);
    let first = common::render(30, 5, |frame| {
        frame.render_widget(&voltage, frame.area());
    });
    let second = common::render(30, 5, |frame| {
        frame.render_widget(&voltage, frame.area());
    });

    assert_eq!(first, second);
    assert_eq!(
        voltage.voltages.voltage_read_points,
        [("Vcore".into(), 1.25), ("in1".into(), 3.3)]
    );
}

#[test]
fn borrowed_rendering_preserves_the_readings_and_label_allocations() {
    let (_guard, voltage) = widget(&[("Vcore", 1.25), ("in1", 3.3)]);
    let readings_ptr = voltage.voltages.voltage_read_points.as_ptr();
    let capacity = voltage.voltages.voltage_read_points.capacity();
    let label_ptrs: Vec<_> = voltage
        .voltages
        .voltage_read_points
        .iter()
        .map(|(label, _)| label.as_ptr())
        .collect();
    let mut terminal = Terminal::new(TestBackend::new(30, 5)).expect("test terminal");

    for _ in 0..3 {
        // Passing a shared reference also guards the borrowed Widget API at
        // compile time. The owned model remains available after every draw.
        terminal
            .draw(|frame| frame.render_widget(&voltage, frame.area()))
            .expect("draw borrowed widget");

        assert_eq!(voltage.voltages.voltage_read_points.as_ptr(), readings_ptr);
        assert_eq!(voltage.voltages.voltage_read_points.capacity(), capacity);
        for ((label, _), &original_ptr) in
            voltage.voltages.voltage_read_points.iter().zip(&label_ptrs)
        {
            assert_eq!(label.as_ptr(), original_ptr);
        }
    }
}

#[test]
fn in_place_voltage_changes_appear_on_the_next_frame() {
    let (_guard, mut voltage) = widget(&[("Vcore", 1.25), ("in1", 3.3)]);
    let readings_ptr = voltage.voltages.voltage_read_points.as_ptr();
    let label_ptr = voltage.voltages.voltage_read_points[0].0.as_ptr();
    let mut terminal = Terminal::new(TestBackend::new(30, 5)).expect("test terminal");

    for value in [1.25, 1.375, 0.0, -1.0] {
        voltage.voltages.voltage_read_points[0].1 = value;
        terminal
            .draw(|frame| frame.render_widget(&voltage, frame.area()))
            .expect("draw updated widget");

        let text = common::buffer_text(terminal.backend().buffer());
        let vcore_row = text.lines().nth(1).expect("first sensor row");
        assert_eq!(
            vcore_row.trim_matches('│').trim(),
            format!("Vcore: {value}V")
        );
        assert!(
            text.contains("in1: 3.3V"),
            "unmodified reading disappeared:\n{text}"
        );
        assert_eq!(voltage.voltages.voltage_read_points[0].1, value);
        assert_eq!(voltage.voltages.voltage_read_points.as_ptr(), readings_ptr);
        assert_eq!(
            voltage.voltages.voltage_read_points[0].0.as_ptr(),
            label_ptr
        );
    }

    // Removing a reading must also clear its old text on the same terminal.
    voltage.voltages.voltage_read_points.pop();
    terminal
        .draw(|frame| frame.render_widget(&voltage, frame.area()))
        .expect("draw removed reading");
    let text = common::buffer_text(terminal.backend().buffer());
    assert!(
        !text.contains("in1:"),
        "removed reading remained on screen:\n{text}"
    );
}

#[test]
fn rendering_reflects_updated_voltage_readings() {
    let (_guard, mut voltage) = widget(&[("old rail", 5.0)]);
    let first = common::buffer_text(&common::render(30, 5, |frame| {
        frame.render_widget(&voltage, frame.area());
    }));
    assert!(first.contains("old rail: 5V"));

    voltage.voltages.voltage_read_points = vec![("new rail".into(), 12.0)];
    let second = common::buffer_text(&common::render(30, 5, |frame| {
        frame.render_widget(&voltage, frame.area());
    }));
    assert!(second.contains("new rail: 12V"));
    assert!(!second.contains("old rail"));
}

#[test]
fn voltage_widget_only_draws_inside_its_assigned_area() {
    let (_guard, voltage) = widget(&[("Vcore", 1.25)]);
    let area = Rect::new(6, 3, 24, 5);
    let buffer = common::render(40, 12, |frame| {
        frame.render_widget(&voltage, area);
    });
    assert!(common::buffer_text(&buffer).contains("Vcore: 1.25V"));

    for y in 0..12 {
        for x in 0..40 {
            if x < area.x || x >= area.right() || y < area.y || y >= area.bottom() {
                assert_eq!(
                    buffer[(x, y)].symbol(),
                    " ",
                    "drew outside the widget at ({x}, {y})"
                );
            }
        }
    }
}

#[test]
fn voltage_widget_clips_readings_to_the_available_height() {
    let (_guard, voltage) = widget(&[
        ("first rail", 1.25),
        ("second rail", 3.3),
        ("third rail", 5.0),
    ]);
    let text = common::buffer_text(&common::render(30, 3, |frame| {
        frame.render_widget(&voltage, frame.area());
    }));

    assert!(text.contains("first rail: 1.25V"));
    assert!(!text.contains("second rail"));
    assert!(!text.contains("third rail"));
    assert!(
        text.contains('└') && text.contains('┘'),
        "readings overwrote the bottom border:\n{text}"
    );
}

#[test]
fn voltage_widget_handles_empty_and_tiny_areas() {
    let (_guard, voltage) = widget(&[("Vcore", 1.25)]);
    for (width, height) in [(0, 0), (0, 3), (3, 0), (1, 1), (2, 2), (4, 3)] {
        let _ = common::render(20, 10, |frame| {
            frame.render_widget(&voltage, Rect::new(2, 2, width, height));
        });
    }
}
