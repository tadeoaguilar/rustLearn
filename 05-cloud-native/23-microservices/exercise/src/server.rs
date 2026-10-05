//! Starting services on local ports (tests, demo). PROVIDED.

use crate::inventory::InventoryService;
use crate::orders::OrdersService;
use crate::pb::inventory_server::InventoryServer;
use crate::pb::orders_server::OrdersServer;
use tokio::sync::oneshot;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Endpoint, Server};

/// A server on 127.0.0.1:<random port>; stopped when `stop` is called or
/// this is dropped.
pub struct Running {
    pub url: String,
    stop: Option<oneshot::Sender<()>>,
}

impl Running {
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn spawn(router: tonic::transport::server::Router) -> Running {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = oneshot::channel::<()>();
    tokio::spawn(
        router.serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
            let _ = rx.await;
        }),
    );
    Running {
        url,
        stop: Some(tx),
    }
}

pub async fn spawn_inventory(service: InventoryService) -> Running {
    spawn(Server::builder().add_service(InventoryServer::new(service))).await
}

pub async fn spawn_orders(service: OrdersService) -> Running {
    spawn(Server::builder().add_service(OrdersServer::new(service))).await
}

/// A channel that connects on first use (and reconnects after failures).
pub fn lazy_channel(url: &str) -> Channel {
    Endpoint::from_shared(url.to_string())
        .expect("valid url")
        .connect_lazy()
}
