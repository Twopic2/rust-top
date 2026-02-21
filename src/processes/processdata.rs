use std::sync::Arc;

use sysinfo::{System};

#[derive(Clone, Debug)]
pub struct CollectProcessData {
    pub pid: u32,
    pub command: String,
    pub program: String,
    pub mem_usage_percent: f32,
    pub cpu_usage_percent: f32,
    pub uid: Option<libc::uid_t>,
    pub user: &'static str,
}

impl CollectProcessData {
    pub fn new() -> Self {
        Self {
            pid: 0,
            command: String::new(),
            program: String::new(),
            mem_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
            uid: None,
            user: ""
        }
    }
    pub fn process_data(&mut self, sys: &mut System) -> Vec<CollectProcessData> {
        sys.refresh_all();

        let mut process_data_vec: Vec<CollectProcessData> = Vec::new();
        let processes = sys.processes();

        for (pid, process) in processes {
            let commands = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");

            let user_id = process.user_id().map(|uid| **uid);

            let user_ = uid_to_user(user_id);

            let data = CollectProcessData {
                pid: pid.as_u32(),
                command: commands,
                program: process.name().to_string_lossy().to_string(),
                mem_usage_percent: (process.memory() as f32 / sys.total_memory() as f32) * 100.0,
                cpu_usage_percent:  process.cpu_usage(),
                uid: user_id,
                user: user_,
            };

            process_data_vec.push(data);
        }

        process_data_vec
    }
}

fn uid_to_user(uid: Option<libc::uid_t>) -> &'static str {
    match uid {
        Some(1000) => "Base User",
        Some(0) => "Root User",
        None => "Idk what's up",
        Some(_) => "Other User",
    }
}
