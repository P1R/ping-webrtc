use futures::StreamExt;
use litep2p::{
    config::ConfigBuilder,
    //protocol::libp2p::ping,
    protocol::{libp2p::ping, request_response},
    transport::{
        quic::config::Config as QuicConfig,
        tcp::config::Config as TcpConfig,
    },
    Litep2p, ProtocolName,
};

// simple example which enables `/ipfs/ping/1.0.0` and `/request/1` protocols
// and TCP and QUIC transports and starts polling events
#[tokio::main]
async fn main() {
    // enable IPFS PING protocol
    let (ping_config, mut ping_event_stream) = ping::Config::default();

    // enable `/request/1` request-response protocol
    let (req_resp_config, mut req_resp_handle) =
        request_response::ConfigBuilder::new(ProtocolName::from("/request/1")).with_max_size(1024).build();

    // build `Litep2pConfig` object
    let config = ConfigBuilder::new()
        .with_tcp(TcpConfig {
            listen_addresses: vec![
                "/ip6/::1/tcp/0".parse().expect("valid address"),
                "/ip4/127.0.0.1/tcp/0".parse().expect("valid address"),
            ],
            ..Default::default()
        })
        .with_quic(QuicConfig {
            listen_addresses: vec!["/ip4/127.0.0.1/udp/0/quic-v1".parse().expect("valid address")],
            ..Default::default()
        })
        .with_libp2p_ping(ping_config)
        .with_request_response_protocol(req_resp_config)
        .build();

    // build `Litep2p` object
    let mut litep2p = Litep2p::new(config).unwrap();

    loop {
        tokio::select! {
            _event = litep2p.next_event() => {},
            _event = req_resp_handle.next() => {},
            _event = ping_event_stream.next() => {},
        }
    }
}
