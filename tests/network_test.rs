//! `data::network` — counters come back as two-element vectors that the graph
//! indexes positionally, so the length contract matters.

use rust_top::data::network::NetworkHarvester;

#[test]
fn curr_network_data_is_rx_then_tx() {
    let mut h = NetworkHarvester::default();
    let curr = h.get_curr_network_data();
    assert_eq!(
        curr.len(),
        2,
        "network_graph.rs indexes [0] and [1]; a shorter vec would panic"
    );
}

#[test]
fn total_network_data_is_rx_then_tx() {
    let mut h = NetworkHarvester::default();
    let total = h.get_total_network_data();
    assert_eq!(total.len(), 2);
}

#[test]
fn totals_never_go_backwards() {
    let mut h = NetworkHarvester::default();
    let first = h.get_total_network_data();
    let second = h.get_total_network_data();
    assert!(second[0] >= first[0], "total rx went backwards");
    assert!(second[1] >= first[1], "total tx went backwards");
}

#[test]
fn repeated_polling_stays_within_the_same_shape() {
    let mut h = NetworkHarvester::default();
    for _ in 0..5 {
        assert_eq!(h.get_curr_network_data().len(), 2);
        assert_eq!(h.get_total_network_data().len(), 2);
    }
}

#[test]
fn ip_address_is_one_of_the_two_known_forms() {
    let mut h = NetworkHarvester::default();
    let ip = h.get_ip_adress();
    assert!(
        ip.starts_with("Local IP address: ") || ip == "Unable to get local IP address",
        "unexpected ip string: {ip:?}"
    );
}

#[test]
fn ip_address_string_is_never_empty() {
    let mut h = NetworkHarvester::default();
    // It becomes a block title; an empty string would silently drop the border label.
    assert!(!h.get_ip_adress().is_empty());
}
