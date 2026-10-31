use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Default)]
pub struct CpuPowerUsage {
    cpu_package_watts: Option<f64>,
}

impl CpuPowerUsage {
    pub fn update(&mut self, watts: Option<f64>) {
        self.cpu_package_watts = watts;
    }
}

impl Widget for &CpuPowerUsage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = match self.cpu_package_watts {
            Some(watts) => format!("Cpu Package {watts:.1} W"),
            None => "Cpu Package N/A".to_string(),
        };

        Paragraph::new(text)
            .block(Block::bordered().title("CPU Power"))
            .render(area, buf);
    }
}
