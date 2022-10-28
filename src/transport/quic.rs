// use std::net::SocketAddr;
// use std::sync::Arc;
//
// use async_trait::async_trait;
// use quinn::{Endpoint, OpenUni};
// use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
//
// use crate::config::{TcpConfig, TransportConfig};
// use crate::transport::{AddrMaybeCached, SocketOpts, Transport};
//
// #[derive(Debug)]
// pub struct QuicTransport {}
//
// #[async_trait]
// impl Transport for QuicTransport {
//     type Acceptor = Endpoint;
//     type Stream = OpenUni;
//     type RawStream = OpenUni;
//
//     fn new(config: &TransportConfig) -> anyhow::Result<Self> where Self: Sized {
//         todo!()
//     }
//
//     fn hint(conn: &Self::Stream, opts: SocketOpts) {
//         todo!()
//     }
//
//     async fn bind<T: ToSocketAddrs + Send + Sync>(&self, addr: T) -> anyhow::Result<Self::Acceptor> {
//         todo!()
//     }
//
//     async fn accept(&self, a: &Self::Acceptor) -> anyhow::Result<(Self::RawStream, SocketAddr)> {
//         todo!()
//     }
//
//     async fn handshake(&self, conn: Self::RawStream) -> anyhow::Result<Self::Stream> {
//         todo!()
//     }
//
//     async fn connect(&self, addr: &AddrMaybeCached) -> anyhow::Result<Self::Stream> {
//         todo!()
//     }
// }


use quinn::{ClientConfig, Endpoint, RecvStream, SendStream, ServerConfig};
use std::{error::Error, net::SocketAddr, sync::Arc,};
use std::io::Result as NetResult;
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use crate::config::SkipServerVerification;



pub fn make_client_endpoint(
    bind_addr: SocketAddr,
) -> Result<Endpoint, Box<dyn Error>> {
    let client_cfg = configure_client()?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_cfg);
    Ok(endpoint)
}


pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<((Endpoint, quinn::Incoming), Vec<u8>), Box<dyn Error>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}


fn configure_client() -> Result<ClientConfig, Box<dyn Error>> {
    // 取消对证书的验证
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    Ok(ClientConfig::new(Arc::new(crypto)))
}

fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(0_u8.into());

    Ok((server_config, cert_der))
}

#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];


pub struct QuicConn{
    pub send: SendStream,
    pub recv: RecvStream,
}

impl QuicConn{
    pub fn new(send: SendStream,recv: RecvStream) -> Self {
        QuicConn{
            send,
            recv
        }
    }
}

impl  tokio::io::AsyncRead for QuicConn {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<NetResult<()>> {
        Pin::new(&mut self.recv).poll_read(cx, buf)
    }
}




impl tokio::io::AsyncWrite for QuicConn {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<NetResult<usize>> {
        Pin::new(&mut self.send).poll_write(cx, buf)
    }

    #[inline]
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<NetResult<usize>> {
        Pin::new(&mut self.send).poll_write_vectored(cx, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.send.is_write_vectored()
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<NetResult<()>> {
        Pin::new(&mut self.send).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<NetResult<()>> {
        Pin::new(&mut self.send).poll_shutdown(cx)
    }
}
