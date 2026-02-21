use std::collections::HashMap;

#[cfg(target_os = "macos")]
use sysinfo::Components;

pub struct TempData;

impl TempData {
    #[cfg(target_os = "macos")]
    pub fn all_temps_mac() -> Option<HashMap<String, Option<f32>>> {
        let mut components = Components::new_with_refreshed_list();
        components.refresh(true);
        let mut temp_map: HashMap<String, Option<f32>> = HashMap::new();

        for comp in components.list_mut() {
            let comp_str = comp.label().to_string();
            temp_map.insert(comp_str, comp.temperature());
        }

        if temp_map.is_empty() {
            None
        } else {
            Some(temp_map)
        }
    }
    
    #[cfg(target_os = "linux")]
    pub fn lm_sensor_temp() -> Option<HashMap<String, Option<f32>>> {
        let sensors = lm_sensors::Initializer::default().initialize().ok()?;
        let mut temp_map: HashMap<String, Option<f32>> = HashMap::new();

        for chip in sensors.chip_iter(None) {
            let chip_label = chip.to_string();

            for feature in chip.feature_iter() {
                let feature_label = feature
                    .label()
                    .ok()
                    .unwrap_or_else(|| "N/A".to_string());

                for sub_feature in feature.sub_feature_iter() {
                    if sub_feature.kind() == Some(lm_sensors::value::Kind::TemperatureInput) {
                        let temp = sub_feature.value().ok().map(|v| v.raw_value() as f32);
                        let comp_label = format!("{}: {}", chip_label, feature_label);
                        temp_map.insert(comp_label, temp);
                    }
                }
            }
        }

        if temp_map.is_empty() {
            None
        } else {
            Some(temp_map)
        }
    }

    pub fn all_temps() -> Option<HashMap<String, Option<f32>>> {
        #[cfg(target_os = "macos")]
        return Self::all_temps_mac();
        #[cfg(target_os = "linux")]
        return Self::lm_sensor_temp();
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        return None;
    }
}
