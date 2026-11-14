use sysinfo::System;
use ratatui::text::Line;

#[cfg(target_os = "macos")]
use crate::data::darwin::cache::CacheMac;

use crate::data::cpu::CpuInfo;
use crate::data::mem::MemInfo;
use crate::data::disk::DiskData;
use crate::data::power::CpuPackageCap;
use crate::data::voltage::Voltage;
use crate::draw::widgets::cpu_graph::{MultiCoreGraph, ColorScheme};
use crate::draw::widgets::cpu_bar::{TotalCoreBar, BarColorScheme};
use crate::draw::widgets::cpu_power::CpuPowerUsage;
use crate::draw::widgets::network_graph::NetworkGraph;
use crate::draw::widgets::process_table::ProcessTable;
use crate::draw::widgets::voltage_bar::VoltageUsage;

pub struct TopCollection {
    pub sys: System,
    pub cpu_model_lines: Line<'static>,
    pub cpu_cache_lines: Vec<Line<'static>>,
    pub mem_lines: Vec<Line<'static>>,
    pub core_graph: MultiCoreGraph,
    pub total_cpu_bar: TotalCoreBar,
    pub cpu_power: CpuPowerUsage,
    cpu_package: Option<CpuPackageCap>,
    pub network_histogram: NetworkGraph,
    pub disk_data: DiskData,
    pub process_tree: ProcessTable,
    pub voltage_bar: VoltageUsage,
}

impl TopCollection {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        let lm_sensors = lm_sensors::Initializer::default().initialize().unwrap();

        let cpu_model_lines = if let Some(cpu_model) = CpuInfo::display_cpu_model(&mut sys) {
            Line::from(cpu_model)
        } else {
            Line::from("Cpu Model isn't here")
        };

        #[cfg(target_os = "macos")]
        let cpu_cache_lines: Vec<Line<'static>> = CacheMac::cache_lines();

        #[cfg(not(target_os = "macos"))]
        let cpu_lines = CpuInfo::display_cpu_cache();
        #[cfg(not(target_os = "macos"))]
        let cpu_cache_lines = cpu_lines.into_iter().map(|(key, value)| {
            Line::from(format!("{}: {} kb", key, value))
        }).collect();

        let mem_lines = MemInfo::display_memory(&mut sys).into_iter().map(|value| {
            Line::from(value)
        }).collect();

        let num_cores = CpuInfo::num_cores(&mut sys);
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);
        let network_histogram = NetworkGraph::new(60);
        let disk_data = DiskData::default();
        let process_tree = ProcessTable::new();
        let cpu_power = CpuPowerUsage::default();
        let cpu_package = CpuPackageCap::new().ok();
        let voltages = Voltage::new(lm_sensors);
        let voltage_bar = VoltageUsage::new(voltages);

        Self {
            sys,
            cpu_model_lines,
            cpu_cache_lines,
            mem_lines,
            core_graph,
            total_cpu_bar,
            cpu_power,
            cpu_package,
            network_histogram,
            disk_data,
            process_tree,
            voltage_bar,
        }
    }

    pub fn refresh_timer(&mut self) {
        CpuInfo::set_refresh_timer(&mut self.sys);
    }

    pub fn update_data(&mut self) {
        let core_usages = CpuInfo::get_core_usages(&mut self.sys);

        for (i, usage) in core_usages.iter().enumerate() {
            self.core_graph.cores.insert(i, *usage);
        }

        self.total_cpu_bar.update(&core_usages);

        let watts = if let Some(package) = self.cpu_package.as_mut() {
            package.refresh().ok().map(|()| package.curr_watt)
        } else {
            self.cpu_package = CpuPackageCap::new().ok();
            None
        };
        self.cpu_power.update(watts);

        self.network_histogram.update();

        self.disk_data.refresh(&mut self.sys);
        self.disk_data.collect_all(&mut self.sys);

        self.voltage_bar.voltages.refresh();

        self.process_tree.refresh(&mut self.sys);
    }
}
