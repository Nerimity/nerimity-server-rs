use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

pub mod pb {
    tonic::include_proto!("socket");
}
use pb::socket_client::SocketClient;

pub fn make_socket_client() -> SocketClient<Channel> {
    let channel = Endpoint::from_static("http://127.0.0.1:50051")
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(5))
        .http2_keep_alive_interval(Duration::from_secs(30))
        .keep_alive_while_idle(true)
        .connect_lazy();

    SocketClient::new(channel)
}
