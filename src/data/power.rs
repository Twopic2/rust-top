/*
    ! Power Capping Framework

    ? Cpu
    * In terms of how Linux monitors power, it would be using RAPL 'Running Average Power Limit' which has two functions
    * Measures energy consumption of major hardware domains using on-die counters
    * Limits sustained power draw by throttling clock frequencies when a domain's rolling average exceeds a configured cap

    CPU package
    └── package-0          ← entire CPU package
        ├── cores          ← CPU cores
        ├── uncore/cache
        ├── memory controller
        └── other package components

*/

/* Credit to https://github.com/level1techs/siomon/blob/main/src/sensors/rapl.rs */

use std::{
    fs::{self, File},
    io::{self, Read, Seek},
    path::Path,
    time::Instant,
};

pub struct CpuPackageCap {
    power_file: File, // energy_uj
    pub curr_watt: f64,
    prev_time: Instant,
    prev_energy: u64,
    max_energy: u64,
}

impl CpuPackageCap {
    pub fn new() -> io::Result<Self> {
        let dir = Path::new("/sys/class/powercap/intel-rapl:0");
        let max_energy = fs::read_to_string(dir.join("max_energy_range_uj"))?
            .trim()
            .parse::<u64>()
            .map_err(io::Error::other)?;
        let mut power_file = File::open(dir.join("energy_uj"))?;
        let prev_energy = Self::read_energy(&mut power_file)?;

        Ok(Self {
            power_file,
            curr_watt: 0.0,
            prev_time: Instant::now(),
            prev_energy,
            max_energy,
        })
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        let energy = Self::read_energy(&mut self.power_file)?;
        let now = Instant::now();
        let time_since_prev = now.duration_since(self.prev_time);

        if time_since_prev.is_zero() {
            return Err(io::Error::other("CPU power sample interval is zero"));
        }

        let delta_uj = if energy >= self.prev_energy {
            energy - self.prev_energy
        } else {
            (self.max_energy - self.prev_energy) + energy
        };

        self.curr_watt = (delta_uj as f64 / 1_000_000.0) / time_since_prev.as_secs_f64();

        self.prev_time = now;
        self.prev_energy = energy;
        Ok(())
    }

    fn read_energy(file: &mut File) -> io::Result<u64> {
        file.rewind()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents.trim().parse::<u64>().map_err(io::Error::other)
    }
}
