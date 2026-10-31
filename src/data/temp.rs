use std::collections::HashMap;
use sysinfo::Components;

pub struct TempData;

/*
    ! Temp notes 
    * k10temp is a Linux kernel driver that allows reading the internal temperature sensor of AMD processors from Family 10h to Family 18h. 
    * It provides temperature readings that are used for monitoring and controlling CPU cooling systems
*/

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
