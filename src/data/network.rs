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

        self.curr_rx = self.network.iter().fold(0, |rx, (_, net)| rx + net.received());
        self.curr_tx = self.network.iter().fold(0, |tx, (_, net)| tx + net.transmitted());

        let mut curr_network = Vec::new();
        curr_network.push(self.curr_rx);
        curr_network.push(self.curr_tx);
        
        curr_network
    }

    pub fn get_total_network_data(&mut self) -> Vec<u64> {
        self.network.refresh(true);

        self.total_rx = self.network.iter().fold(0, |rx, (_, net)| rx + net.total_received());
        self.total_tx = self.network.iter().fold(0, |tx, (_, net)| tx + net.total_transmitted());

        let mut curr_network = Vec::new();
        curr_network.push(self.total_rx);
        curr_network.push(self.total_tx);
        
        curr_network
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
