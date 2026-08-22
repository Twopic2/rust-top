//! `data::temp` — hardware sensors. Availability varies per machine and inside
//! containers, so every test tolerates `None` and only checks the shape.

use rust_top::data::temp::TempData;

#[test]
fn all_temps_is_none_or_a_non_empty_map() {
    match TempData::all_temps() {
        None => {} // no sensors exposed on this host: valid
        Some(map) => assert!(
            !map.is_empty(),
            "all_temps must return None rather than an empty map"
        ),
    }
}

#[test]
fn sensor_labels_are_non_empty() {
    let Some(map) = TempData::all_temps() else {
        return;
    };
    for label in map.keys() {
        assert!(!label.trim().is_empty(), "found a blank sensor label");
    }
}

#[test]
fn reported_temperatures_are_physically_plausible() {
    let Some(map) = TempData::all_temps() else {
        return;
    };
    for (label, temp) in &map {
        let Some(t) = temp else { continue };
        assert!(t.is_finite(), "{label} reported a non-finite temperature");
        assert!(
            (-100.0..=250.0).contains(t),
            "{label} reported an implausible {t}°C"
        );
    }
}

#[test]
fn availability_is_consistent_across_calls() {
    // The draw layer calls all_temps() to decide layout height and then again
    // to render; those two calls must agree or the panel renders into a
    // zero-height area.
    assert_eq!(
        TempData::all_temps().is_some(),
        TempData::all_temps().is_some()
    );
}

#[test]
fn label_set_is_stable_across_calls() {
    let (Some(a), Some(b)) = (TempData::all_temps(), TempData::all_temps()) else {
        return;
    };
    let mut ka: Vec<_> = a.keys().collect();
    let mut kb: Vec<_> = b.keys().collect();
    ka.sort();
    kb.sort();
    assert_eq!(ka, kb, "sensor label set changed between calls");
}
