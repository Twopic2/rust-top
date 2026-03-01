use sysinfo::Networks;
use local_ip_address::local_ip;

#[derive(Default)]
pub struct NetworkHarvester {
    curr_rx: u64,
    curr_tx: u64,
    total_rx: u64,
    total_tx: u64,
    network: Networks,
}

impl NetworkHarvester {
    pub fn get_curr_network_data(&mut self) -> Vec<u64> {
        self.network.refresh(true);

        self.curr_rx = 0;
        self.curr_tx = 0;

        for (_, netwrk) in self.network.iter() {
            self.curr_rx += netwrk.received();
            self.curr_tx += netwrk.transmitted();
        }

        vec![self.curr_rx, self.curr_tx]
    }

    pub fn get_total_network_data(&mut self) -> Vec<u64> {
        self.network.refresh(true);

        self.total_rx = 0;
        self.total_tx = 0;

        for (_, netwrk) in self.network.iter() {
            self.total_rx += netwrk.total_received();
            self.total_tx += netwrk.total_transmitted();
        }

        vec![self.total_rx, self.total_tx]
    }
    
    pub fn get_ip_adress(&mut self) -> String {
        let local_ip = local_ip();

        if let Ok(local_ip) = local_ip {
            format!("Local IP address: {}", local_ip)
        } else {
            String::from("Unable to get local IP address")
        }
    }
}
