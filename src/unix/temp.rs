use std::collections::HashMap;

use sysinfo::Components;
pub struct TempData {
    components: Components,
}

impl TempData {
    pub fn new() -> Self {
        let components = Components::new_with_refreshed_list();
        Self { 
            components 
        }
    }

    pub fn get_all_temps(&mut self) -> Option<HashMap<String, Option<f32>>> {
        self.components.refresh(true);
        let mut temp_map: HashMap<String, Option<f32>> = HashMap::new();
        
        for comp in self.components.list_mut() {
            let comp_str = comp.label().to_string();

            if let Some(_temperature) = comp.temperature() {
                temp_map.insert(comp_str, comp.temperature());
            } else {
                temp_map.insert(comp_str, None); 
            }
        }
        Some(temp_map)
    }
}