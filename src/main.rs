use litep2p::{
    config::ConfigBuilder,
    protocol::{
        libp2p::ping::{Config as PingConfig, PingEvent},
        mdns::{Config as MdnsConfig, MdnsEvent},
    },
    transport::{
        webrtc::config::Config as WebRTCConfig,
    },
    Litep2p,
};

use futures::{Stream, StreamExt};

use std::time::Duration;

/// simple event loop which discovers peers over mDNS,
/// establishes a connection to them and calculates the PING time
async fn peer_event_loop(
    mut litep2p: Litep2p,
    mut ping_event_stream: Box<dyn Stream<Item = PingEvent> + Send + Unpin>,
    mut mdns_event_stream: Box<dyn Stream<Item = MdnsEvent> + Send + Unpin>,
) {
    loop {
        tokio::select! {
            _ = litep2p.next_event() => {}
            event = ping_event_stream.next() => match event.unwrap() {
                PingEvent::Ping { peer, ping } => {
                    println!("ping received from {peer:?}: {ping:?}");
                }
            },
            event = mdns_event_stream.next() => match event.unwrap() {
                MdnsEvent::Discovered(addresses) => {
                    litep2p.dial_address(addresses[0].clone()).await.unwrap();
                }
            }
        }
    }
}

/// helper function for creating `Litep2p` object
fn make_litep2p() -> (
    Litep2p,
    Box<dyn Stream<Item = PingEvent> + Send + Unpin>,
    Box<dyn Stream<Item = MdnsEvent> + Send + Unpin>,
) {
    // initialize IPFS ping and mDNS
    let (ping_config, ping_event_stream) = PingConfig::default();
    let (mdns_config, mdns_event_stream) = MdnsConfig::new(Duration::from_secs(30));
    let webrtc_config = WebRTCConfig::default();

    // build `Litep2p`, passing in configurations for IPFS and mDNS
    //let litep2p_config = ConfigBuilder::new()
    //    // `litep2p` will bind to `/ip6/::1/tcp/0` by default
    //    .with_tcp(Default::default())
    //    .with_libp2p_ping(ping_config)
    //    .with_mdns(mdns_config)
    //    .build();
    
    // build `Litep2p`, passing in configurations for IPFS and mDNS
    let litep2p_config = ConfigBuilder::new()
        // `litep2p` will bind to `/ip6/::1/tcp/0` by default
        .with_tcp(Default::default())
        .with_libp2p_ping(ping_config)
        .with_mdns(mdns_config)
        .with_webrtc(webrtc_config)
        .build();

    // build `Litep2p` and return it + event streams
    (
        Litep2p::new(litep2p_config).unwrap(),
        ping_event_stream,
        mdns_event_stream,
    )
}

#[tokio::main]
async fn main() {
    // initialize `Litep2p` objects for the peers
    let (litep2p1, ping_event_stream1, mdns_event_stream1) = make_litep2p();
    let (litep2p2, ping_event_stream2, mdns_event_stream2) = make_litep2p();

    // starts separate tasks for the first and second peer
    tokio::spawn(peer_event_loop(
        litep2p1,
        ping_event_stream1,
        mdns_event_stream1,
    ));
    tokio::spawn(peer_event_loop(
        litep2p2,
        ping_event_stream2,
        mdns_event_stream2,
    ));

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
