use sysinfo::System;
use ratatui::text::Line;

#[cfg(target_os = "macos")]
use crate::data::darwin::cache::CacheMac;

use crate::data::cpu::CpuInfo;
use crate::data::mem::MemInfo;
use crate::data::disk::DiskData;
use crate::draw::widgets::cpu_graph::{MultiCoreGraph, ColorScheme};
use crate::draw::widgets::cpu_bar::{TotalCoreBar, TempBar, BarColorScheme};
use crate::draw::widgets::network_graph::NetworkGraph;
use crate::draw::widgets::process_table::ProcessTable;

pub struct TopCollection {
    pub sys: System,
    pub cpu_model_lines: Line<'static>,
    pub cpu_cache_lines: Vec<Line<'static>>,
    pub mem_lines: Vec<Line<'static>>,
    pub core_graph: MultiCoreGraph,
    pub total_cpu_bar: TotalCoreBar,
    pub temp_bar: TempBar,
    pub network_histogram: NetworkGraph,
    pub disk_data: DiskData,
    pub process_tree: ProcessTable,
}

impl TopCollection {
    pub fn new() -> Self {
        let mut sys = System::new_all();

        let cpu_model_lines = if let Some(cpu_model) = CpuInfo::display_cpu_model(&mut sys) {
            Line::from(cpu_model)
        } else {
            Line::from("Cpu Model isn't here")
        };

        #[cfg(target_os = "macos")]
        let cpu_cache_lines: Vec<Line<'static>> = CacheMac::cache_lines();

        #[cfg(not(target_os = "macos"))]
        let cpu_lines = CpuInfo::display_cpu_cache();
        let cpu_cache_lines = cpu_lines
            .into_iter().map(|(key, value)| {
                Line::from(format!("{}: {} kb", key, value))
            }).collect();

        let mem_lines = MemInfo::display_memory(&mut sys).into_iter().map(|value| {
            Line::from(value)
        }).collect();

        let num_cores = CpuInfo::num_cores(&mut sys);
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);
        let temp_bar = TempBar::new(BarColorScheme::Green);
        let network_histogram = NetworkGraph::new(60);
        let disk_data = DiskData::default();
        let process_tree = ProcessTable::new();

        Self {
            sys,
            cpu_model_lines,
            cpu_cache_lines,
            mem_lines,
            core_graph,
            total_cpu_bar,
            temp_bar,
            network_histogram,
            disk_data,
            process_tree,
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
        self.temp_bar.update();

        self.network_histogram.update();

        self.disk_data.refresh(&mut self.sys);
        self.disk_data.collect_all(&mut self.sys);

        self.process_tree.refresh(&mut self.sys);
    }
}
