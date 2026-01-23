use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Chart, Dataset, Axis, GraphType},
    style::{Color, Style, Modifier},
    symbols,
    text::Span,
};
use crate::unix::network::NetworkHarvester;

const GIGABYTE: f64 = 1024.0 * 1024.0 * 1024.0;
const MEGABYTE: f64 = 1024.0 * 1024.0;
const KILOBYTE: f64 = 1024.0;

pub struct NetworkHistogram {
    rx_history: Vec<(f64, f64)>,
    tx_history: Vec<(f64, f64)>,
    total_rx: u64,
    total_tx: u64,
    max_points: usize,
    harvester: NetworkHarvester,
    tick: f64,
}

impl NetworkHistogram {
    pub fn new(max_points: usize) -> Self {
        Self {
            rx_history: Vec::with_capacity(max_points),
            tx_history: Vec::with_capacity(max_points),
            total_rx: 0,
            total_tx: 0,
            max_points,
            harvester: NetworkHarvester::init(),
            tick: 0.0,
        }
    }

    pub fn update(&mut self) {
        let curr = self.harvester.get_curr_network_data();
        let total = self.harvester.get_total_network_data();

        self.rx_history.push((self.tick, curr[0] as f64));
        self.tx_history.push((self.tick, curr[1] as f64));
        self.total_rx = total[0];
        self.total_tx = total[1];
        self.tick += 1.0;

        if self.rx_history.len() > self.max_points {
            self.rx_history.remove(0);
        }
        if self.tx_history.len() > self.max_points {
            self.tx_history.remove(0);
        }
    }

    fn format_rate(bytes: f64) -> String {
        if bytes >= GIGABYTE {
            format!("{:.1}Gb/s", bytes / GIGABYTE)
        } else if bytes >= MEGABYTE {
            format!("{:.1}Mb/s", bytes / MEGABYTE)
        } else if bytes >= KILOBYTE {
            format!("{:.1}Kb/s", bytes / KILOBYTE)
        } else {
            format!("{:.0}b/s", bytes)
        }
    }

    fn format_total(bytes: u64) -> String {
        if bytes as f64 >= GIGABYTE {
            format!("{:.1}GB", bytes as f64 / GIGABYTE)
        } else if bytes as f64 >= MEGABYTE {
            format!("{:.1}MB", bytes as f64 / MEGABYTE)
        } else if bytes as f64 >= KILOBYTE {
            format!("{:.1}KB", bytes as f64 / KILOBYTE)
        } else {
            format!("{}B", bytes)
        }
    }

    fn format_axis_label(bytes: f64) -> String {
        if bytes >= GIGABYTE {
            format!("{:.1}G", bytes / GIGABYTE)
        } else if bytes >= MEGABYTE {
            format!("{:.1}M", bytes / MEGABYTE)
        } else if bytes >= KILOBYTE {
            format!("{:.1}K", bytes / KILOBYTE)
        } else if bytes > 0.0 {
            format!("{:.0}", bytes)
        } else {
            "0".to_string()
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let current_rx = self.rx_history.last().map(|(_, v)| *v).unwrap_or(0.0);
        let current_tx = self.tx_history.last().map(|(_, v)| *v).unwrap_or(0.0);

        let max_rx = self.rx_history.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
        let max_tx = self.tx_history.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
        let max_val = max_rx.max(max_tx).max(KILOBYTE); 

        let x_min = self.rx_history.first().map(|(t, _)| *t).unwrap_or(0.0);
        let x_max = self.tick.max(x_min + 1.0);

        let datasets = vec![
            Dataset::default()
                .name(format!(
                    "RX: {} All: {}",
                    Self::format_rate(current_rx),
                    Self::format_total(self.total_rx)
                ))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&self.rx_history),
            Dataset::default()
                .name(format!(
                    "TX: {} All: {}",
                    Self::format_rate(current_tx),
                    Self::format_total(self.total_tx)
                ))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.tx_history),
        ];

        let y_labels = vec![
            Span::raw("0"),
            Span::raw(Self::format_axis_label(max_val / 2.0)),
            Span::raw(Self::format_axis_label(max_val)),
        ];

        let time_span = (self.max_points as f64 * 2.0) as i32; 
        let x_labels = vec![
            Span::raw(format!("{}s", time_span)),
            Span::raw(format!("{}s", time_span / 2)),
            Span::raw("0s"),
        ];

        let ip_address = self.harvester.get_ip_adress();

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(ip_address)
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::DarkGray))
                    .labels(x_labels)
                    .bounds([x_min, x_max]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::DarkGray))
                    .labels(y_labels)
                    .bounds([0.0, max_val]),
            );

        frame.render_widget(chart, area);
    }
}
