use sysinfo::Components;
pub struct TempData {
    components: Components,
}

impl TempData {
    pub fn new() -> Self {
        let components = Components::new_with_refreshed_list();
        Self { components }
    }

    pub fn get_cpu_temp(&mut self) -> Option<f32> {
        self.components.refresh(true);
        
        for comp in self.components.list_mut() {
            comp.refresh();

            if let Some(temp) = comp.temperature() {
                return Some(temp);
            }
        }
        None
    }
}