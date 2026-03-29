use crate::models::{NetworkInterface, NetworkMetrics};
use sysinfo::Networks;

pub fn collect_network_metrics() -> NetworkMetrics {
    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    for (name, data) in &networks {
        let rx = data.total_received();
        let tx = data.total_transmitted();
        total_rx += rx;
        total_tx += tx;

        interfaces.push(NetworkInterface {
            name: name.clone(),
            received_bytes: rx,
            transmitted_bytes: tx,
            received_packets: data.total_packets_received(),
            transmitted_packets: data.total_packets_transmitted(),
        });
    }

    // Sort by total traffic
    interfaces.sort_by(|a, b| {
        (b.received_bytes + b.transmitted_bytes).cmp(&(a.received_bytes + a.transmitted_bytes))
    });

    NetworkMetrics {
        interfaces,
        total_received_bytes: total_rx,
        total_transmitted_bytes: total_tx,
    }
}
