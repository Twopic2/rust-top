use sysinfo::Disks;
use sysinfo::System;

/* implamentation inspired by Bottom 
https://github.com/ClementTsang/bottom
*/
#[derive(Debug)]
pub struct DiskData {
    sys: System,
    disks: Disks,
    disk_name: String,
    filesytem: String,
    mount: String,
    total: u64,
    available: u64,
    read: u64,
    write: u64,
}

impl DiskData {
    pub fn new() -> Self {
        let sys = System::new_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            disk_name: String::new(),
            filesytem: String::new(),
            mount: String::new(),
            total: 0,
            available: 0,
            read: 0,
            write: 0,
        }
    }

    pub fn refresh(&mut self) {
        self.disks.refresh(true);
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    }

    pub fn get_disk(&mut self) -> &str {
        if let Some(disk) = self.disks.list().first() {
            self.disk_name = disk.name().to_string_lossy().to_string();
        }
        &self.disk_name
    }

    pub fn get_filesystem(&mut self) -> &str {
        if let Some(disk) = self.disks.list().first() {
            self.filesytem = disk.file_system().to_string_lossy().to_string();
        }
        &self.filesytem
    }

    pub fn get_mount(&mut self) -> &str {
        if let Some(disk) = self.disks.list().first() {
            self.mount = disk.mount_point().to_string_lossy().to_string();
        }
        &self.mount
    }

    pub fn get_total(&mut self) -> u64 {
        if let Some(disk) = self.disks.list().first() {
            self.total = disk.total_space();
        }
        self.total
    }

    pub fn get_available(&mut self) -> u64 {
        if let Some(disk) = self.disks.list().first() {
            self.available = disk.available_space();
        }
        self.available
    }

    pub fn get_read(&mut self) -> u64 {
        self.read = self.sys.processes()
            .values()
            .map(|p| p.disk_usage().total_read_bytes)
            .sum();
        self.read
    }

    pub fn get_write(&mut self) -> u64 {
        self.write = self.sys.processes()
            .values()
            .map(|p| p.disk_usage().total_written_bytes)
            .sum();
        self.write
    }
}
