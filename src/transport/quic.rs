use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use quinn::{Endpoint, OpenUni};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::config::{TcpConfig, TransportConfig};
use crate::transport::{AddrMaybeCached, SocketOpts, Transport};

#[derive(Debug)]
pub struct QuicTransport {}

#[async_trait]
impl Transport for QuicTransport {
    type Acceptor = Endpoint;
    type Stream = OpenUni;
    type RawStream = OpenUni;

    fn new(config: &TransportConfig) -> anyhow::Result<Self> where Self: Sized {
        todo!()
    }

    fn hint(conn: &Self::Stream, opts: SocketOpts) {
        todo!()
    }

    async fn bind<T: ToSocketAddrs + Send + Sync>(&self, addr: T) -> anyhow::Result<Self::Acceptor> {
        todo!()
    }

    async fn accept(&self, a: &Self::Acceptor) -> anyhow::Result<(Self::RawStream, SocketAddr)> {
        todo!()
    }

    async fn handshake(&self, conn: Self::RawStream) -> anyhow::Result<Self::Stream> {
        todo!()
    }

    async fn connect(&self, addr: &AddrMaybeCached) -> anyhow::Result<Self::Stream> {
        todo!()
    }
}