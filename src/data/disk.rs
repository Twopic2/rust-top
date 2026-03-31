use sysinfo::{Disks, System};

/* implamentation inspired by Bottom
https://github.com/ClementTsang/bottom
*/
#[derive(Debug, Default)]
pub struct DiskData {
    pub disks: Disks,
    pub disk_name: Vec<String>,
    pub filesytem: Vec<String>,
    pub mount: Vec<String>,
    pub total: Vec<u64>,
    pub available: Vec<u64>,
    pub curr_read: Vec<u64>,
    pub curr_write: Vec<u64>,
}

impl DiskData {
    pub fn refresh(&mut self, sys: &mut System) {
        self.disks.refresh(true);
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
    }
    
    fn get_total_io_read(&self, sys: &mut System) -> u64 {
        sys.processes()
            .values()
            .map(|p| p.disk_usage().read_bytes)
            .sum()
    }

    fn get_total_io_write(&self, sys: &mut System) -> u64 {
        sys.processes()
            .values()
            .map(|p| p.disk_usage().written_bytes)
            .sum()
    }

    pub fn collect_all(&mut self, sys: &mut System) {
        #[cfg(target_os = "macos")]
        let mut disks_data: Vec<_> = self.disks.list().iter()
        .filter(|d| {
            let mount_point = d.mount_point().to_string_lossy();
            let name = d.name().to_string_lossy();
            // Annoying fucking lines when I was in debug mode
            !mount_point.starts_with("/System/Volumes/") &&
            !mount_point.starts_with("/private/var/") &&
            !mount_point.starts_with("/dev") &&
            !name.contains("disk image") &&
            d.total_space() > 0 
        })
        .map(|d| (
            d.name().to_string_lossy().to_string(),
            d.file_system().to_string_lossy().to_string(),
            d.mount_point().to_string_lossy().to_string(),
            d.total_space(),
            d.available_space(),
        )).collect();

        #[cfg(not(target_os = "macos"))]
        let mut disks_data: Vec<_> = self.disks.list().iter()
        .filter(|d| d.name().to_string_lossy().starts_with("/dev/"))
        .map(|d| (
            d.name().to_string_lossy().to_string(),
            d.file_system().to_string_lossy().to_string(),
            d.mount_point().to_string_lossy().to_string(),
            d.total_space(),
            d.available_space(),
        )).collect();

        disks_data.sort_by(|a, b| a.2.cmp(&b.2));

        self.disk_name = disks_data.iter().map(|d| d.0.clone()).collect();
        self.filesytem = disks_data.iter().map(|d| d.1.clone()).collect();
        self.mount = disks_data.iter().map(|d| d.2.clone()).collect();
        self.total = disks_data.iter().map(|d| d.3).collect();
        self.available = disks_data.iter().map(|d| d.4).collect();

        let total_read = self.get_total_io_read(sys);
        let total_write = self.get_total_io_write(sys);

        self.curr_read = vec![total_read; disks_data.len()];
        self.curr_write = vec![total_write; disks_data.len()];
    }
}
