use std::collections::HashMap;
use sysinfo::Components;

pub struct TempData;

impl TempData {
    pub fn all_temps() -> Option<HashMap<String, Option<f32>>> {
        let mut components = Components::new_with_refreshed_list();
        components.refresh(true);
        let mut temp_map: HashMap<String, Option<f32>> = HashMap::new();

        for comp in components.list() {
            temp_map.insert(comp.label().to_string(), comp.temperature());
        }

        if temp_map.is_empty() {
            None
        } else {
            Some(temp_map)
        }
    }
}
