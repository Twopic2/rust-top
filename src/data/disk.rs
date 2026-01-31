use sysinfo::{Disks, System};

/* implamentation inspired by Bottom
https://github.com/ClementTsang/bottom
*/
#[derive(Debug)]
pub struct DiskData {
    sys: System,
    disks: Disks,
    disk_name: Vec<String>,
    filesytem: Vec<String>,
    mount: Vec<String>,
    total: Vec<u64>,
    available: Vec<u64>,
    curr_read: Vec<u64>,
    curr_write: Vec<u64>,
}

impl DiskData {
    pub fn new() -> Self {
        let sys = System::new_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            disk_name: Vec::new(),
            filesytem: Vec::new(),
            mount: Vec::new(),
            total: Vec::new(),
            available: Vec::new(),
            curr_read: Vec::new(),
            curr_write: Vec::new(),
        }
    }

    pub fn refresh(&mut self) {
        #[cfg(not(target_os = "macos"))]
        self.disks.refresh(true);

        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
    }
    
    fn get_total_io_read(&self) -> u64 {
        #[cfg(target_os = "macos")]
        return 0;

        #[cfg(not(target_os = "macos"))]
        self.sys.processes()
            .values()
            .map(|p| p.disk_usage().read_bytes)
            .sum()
    }

    fn get_total_io_write(&self) -> u64 {
        #[cfg(target_os = "macos")]
        return 0;

        #[cfg(not(target_os = "macos"))]
        self.sys.processes()
            .values()
            .map(|p| p.disk_usage().written_bytes)
            .sum()
    }

    pub fn collect_all(&mut self) {
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

        let total_read = self.get_total_io_read();
        let total_write = self.get_total_io_write();

        self.curr_read = vec![total_read; disks_data.len()];
        self.curr_write = vec![total_write; disks_data.len()];
    }

    pub fn get_disks(&self) -> &[String] {
        &self.disk_name
    }

    pub fn get_filesystems(&self) -> &[String] {
        &self.filesytem
    }

    pub fn get_mounts(&self) -> &[String] {
        &self.mount
    }

    pub fn get_totals(&self) -> &[u64] {
        &self.total
    }

    pub fn get_available(&self) -> &[u64] {
        &self.available
    }

    pub fn get_reads(&self) -> &[u64] {
        &self.curr_read
    }

    pub fn get_writes(&self) -> &[u64] {
        &self.curr_write
    }

    pub fn len(&self) -> usize {
        self.disk_name.len()
    }
}
