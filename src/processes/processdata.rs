use std::{fs::File, io::{BufRead, BufReader, ErrorKind}, str::Bytes, sync::Arc, todo};

use std::result::Result::Ok;
use sysinfo::{System};

#[derive(Clone, Debug, Default)]
pub struct Io {
   pub read_bytes: u64,
   pub write_bytes: u64
}

// TODO
/* Might make it where processdata is split between macos/unix/windows */

#[derive(Clone, Debug, Default)]
pub struct CollectProcessData {
    pub pid: u32,
    pub command: String,
    pub program: String,
    pub mem_usage_percent: f32,
    pub cpu_usage_percent: f32,
    // pub parent_pid: Option<u32>
    // pub process_state: &'static str,
    pub disk_io: Option<Io>,
    pub uid: Option<libc::uid_t>, // ! uid_t is u32
    pub user: &'static str, //Might change this to Arc
}

impl CollectProcessData {
    pub fn process_data(&mut self, sys: &mut System) -> Vec<CollectProcessData> {
        let mut process_data_vec: Vec<CollectProcessData> = Vec::new();
        let processes = sys.processes();

        for (pid, process) in processes {
            let commands = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");

            let user_id = process.user_id().map(|uid| **uid);
            let user_ = uid_to_user(user_id);

            let io = pid_io_results(pid.as_u32()).ok();

            let data = CollectProcessData {
                pid: pid.as_u32(),
                command: commands,
                program: process.name().to_string_lossy().to_string(),
                mem_usage_percent: (process.memory() as f32 / sys.total_memory() as f32) * 100.0,
                cpu_usage_percent:  process.cpu_usage() / sys.cpus().len() as f32,
                disk_io: io,
                uid: user_id,
                user: user_,
            };

            if data.cpu_usage_percent > 0.0 || data.mem_usage_percent > 0.0 {
                process_data_vec.push(data);
            }
        }

        process_data_vec
    }

    pub fn process_status(&mut self) -> &'static str {
        todo!()
    }
}

// Credit to ClementTsang inside process.rs
fn pid_io_results(pid: u32) -> anyhow::Result<Io> {
    let io_file = File::open(format!("/proc/{pid}/io"))?;

    let mut reader = BufReader::new(io_file);

    let mut read_bytes = 0;
    let mut write_bytes = 0;

    let mut buffer = String::new();
    while let Ok(n) = reader.read_line(&mut buffer) {
        if n == 0 {
            break;
        }

        if let Some((field, value)) = buffer.split_once(':') {
            match field {
                "read_bytes" => {
                    read_bytes = value.trim().parse().unwrap()
                },
                "write_bytes" => {
                    write_bytes = value.trim().parse().unwrap()
                },
                _ => {
                    buffer.clear();
                    continue;
                 }
            }
        }

        buffer.clear();
    }

    Ok(Io {
        read_bytes,
        write_bytes
    })
}

fn uid_to_user(uid: Option<libc::uid_t>) -> &'static str {
    match uid {
        Some(1000) => "Base User",
        Some(0) => "Root User",
        None => "Idk what's up",
        Some(_) => "Other User",
    }
}
