use sysinfo::System;
use crate::tools::units::GIGABYTE;

pub struct MemInfo;

impl MemInfo {
    pub fn display_memory(sys: &mut System) -> Option<Vec<String>> {
        let mut info = Vec::new();
        let total = sys.total_memory() as f64 / GIGABYTE;
        let used = sys.used_memory() as f64 / GIGABYTE;
        let total_swap = sys.total_swap() as f64 / GIGABYTE;
        let used_swap = sys.used_swap() as f64 / GIGABYTE;
        info.push(format!("Total: {:.2} GB    Total Swap: {:.2} GB", total, total_swap));
        info.push(format!("Used: {:.2} GB    Used Swap: ({:.2} GB)", used, used_swap));
        Some(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_format_total_gb() {
        let bytes: u64 = 16 * 1024 * 1024 * 1024;
        let gb = bytes as f64 / GIGABYTE;
        assert_eq!(format!("{:.2} GB", gb), "16.00 GB");
    }

    #[test]
    fn memory_format_fractional_gb() {
        let bytes: u64 = (6.5 * GIGABYTE) as u64;
        let gb = bytes as f64 / GIGABYTE;
        assert_eq!(format!("{:.2} GB", gb), "6.50 GB");
    }

    #[test]
    fn memory_display_line_format() {
        let total: f64 = 16.0;
        let total_swap: f64 = 8.0;
        let line = format!("Total: {:.2} GB    Total Swap: {:.2} GB", total, total_swap);
        assert_eq!(line, "Total: 16.00 GB    Total Swap: 8.00 GB");
    }

    #[test]
    fn memory_used_line_format() {
        let used: f64 = 5.25;
        let used_swap: f64 = 1.10;
        let line = format!("Used: {:.2} GB    Used Swap: ({:.2} GB)", used, used_swap);
        assert_eq!(line, "Used: 5.25 GB    Used Swap: (1.10 GB)");
    }

    #[test]
    fn display_memory_returns_two_lines() {
        let mut sys = System::new_all();
        sys.refresh_memory();
        let info = MemInfo::display_memory(&mut sys);
        assert!(info.is_some());
        let lines = info.unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Total:"));
        assert!(lines[1].starts_with("Used:"));
    }
}
