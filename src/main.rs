#![feature(async_closure)]

use crate::utils::*;
use crate::visits::{Visit, VisitsLog};
use futures_util::{Stream, TryFutureExt};
use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use lazy_static::lazy_static;
use log::{error, info, LevelFilter};
use rustls::internal::pemfile;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::fs::File;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{convert::Infallible, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

mod blog;
mod shortcuts;
mod static_assets;
mod utils;
mod visits;

lazy_static! {
    static ref ADMIN_KEY: String = std::fs::read_to_string("adminkey.txt")
        .expect("No admin key file found at adminkey.txt.")
        .trim()
        .to_owned();
    static ref HANDLER: Arc<RwLock<Option<RootHandler>>> = Default::default();
}

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Couldn't initialize logging.");
    info!("The admin key is {}.", &ADMIN_KEY.to_owned());

    // Load the config file. It includes the address and certificate information.
    let config = std::fs::read_to_string("Config.toml")
        .unwrap()
        .parse::<toml::Value>()
        .unwrap();
    let address: SocketAddr = config["address"].as_str().unwrap().parse().unwrap();
    let tls_config = config
        .get("certificate")
        .and_then(|it| it.as_table())
        .map(|cert_info| {
            let certs = load_certs(cert_info["certificate"].as_str().unwrap());
            let key = load_private_key(cert_info["key"].as_str().unwrap());
            let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
            cfg.set_single_cert(certs, key).unwrap();
            // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
            cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
            std::sync::Arc::new(cfg)
        });

    // Start the handler. It downloads the newest articles, loads shortcuts etc.
    {
        let mut handler = HANDLER.write().await;
        *handler = Some(RootHandler::new().await);
    }

    // Start the server.
    // TODO: Deduplicate
    if let Some(tls_config) = tls_config {
        let make_service = make_service_fn(|_conn| {
            async move {
                let service = service_fn(|request| {
                    async move {
                        let request = Request::from(&request, &ADMIN_KEY).unwrap(); // TODO
                        info!("Request: {:?}", request);
                        let response = HANDLER
                            .read()
                            .await
                            .as_ref()
                            .unwrap()
                            .handle(&request)
                            .await;
                        let result: Result<Response, Infallible> = Ok(response);
                        result
                    }
                });
                let service = ServiceBuilder::new()
                    .timeout(Duration::from_secs(10))
                    .layer(CompressionLayer::new())
                    .service(service);
                Ok::<_, Infallible>(service)
            }
        });

        let tcp = TcpListener::bind(&address)
            .await
            .expect("Couldn't bind to socket.");
        let tls_acceptor = TlsAcceptor::from(tls_config);
        // Prepare a long-running future stream to accept and serve clients.
        let incoming_tls_stream = async_stream::stream! {
            loop {
                let (socket, _) = tcp.accept().await?;
                let stream = tls_acceptor.accept(socket).map_err(|e| {
                    error!("Voluntary server halt due to client-connection error...");
                    // TODO: Errors should be handled here, instead of server aborting.
                    // Ok(None)
                    std::io::Error::new(std::io::ErrorKind::Other, format!("TLS Error: {:?}", e))
                });
                yield stream.await;
            }
        };
        let server = Server::builder(HyperAcceptor {
            acceptor: Box::pin(incoming_tls_stream),
        })
        .serve(make_service);
        info!("Listening on https://{}", address);
        server.await.expect("Error while running the server.");
    } else {
        // Create a lambda that can create new services on demand, using the handler.
        let make_service = make_service_fn(|_conn| {
            async move {
                let service = service_fn(|request| {
                    async move {
                        let request = Request::from(&request, &ADMIN_KEY).unwrap(); // TODO
                        info!("Request: {:?}", request);
                        let response = HANDLER
                            .read()
                            .await
                            .as_ref()
                            .unwrap()
                            .handle(&request)
                            .await;
                        let result: Result<Response, Infallible> = Ok(response);
                        result
                    }
                });
                let service = ServiceBuilder::new()
                    .timeout(Duration::from_secs(10))
                    .layer(CompressionLayer::new())
                    .service(service);
                Ok::<_, Infallible>(service)
            }
        });

        let server = Server::bind(&address).serve(make_service);
        info!("Listening on http://{}", address);
        server.await.expect("Error while running the server.");
    }
}

// Load public certificate from file.
fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = File::open(filename).expect("Failed to open certfile.");
    let mut reader = std::io::BufReader::new(certfile);
    pemfile::certs(&mut reader).expect("Failed to load the certificate.")
}

// Load private key from file.
fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = std::fs::File::open(filename).expect("Failed to open key file.");
    let mut reader = std::io::BufReader::new(keyfile);
    let keys = pemfile::rsa_private_keys(&mut reader).expect("Failed to load private key.");
    if keys.len() != 1 {
        panic!("Expected a single private key.");
    }
    keys[0].clone()
}

struct HyperAcceptor<'a> {
    acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, std::io::Error>> + 'a>>,
}

impl hyper::server::accept::Accept for HyperAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = std::io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}

/// The public handle function that handles all requests.
///
/// # Visits API
///
/// The visits API looks like this (it's not exactly following the best practices for RESTful
/// APIs, but having all parameters including the key in the querystrings allows for easier
/// deserialization):
///
/// * GET /api/visits: Returns a list of all visits.
struct RootHandler {
    visits: RwLock<VisitsLog>,
    shortcuts: shortcuts::Handler,
    blog: blog::Handler,
}
impl RootHandler {
    async fn new() -> Self {
        Self {
            visits: RwLock::new(VisitsLog::new()),
            shortcuts: shortcuts::Handler::new(),
            blog: blog::Handler::new().await,
        }
    }
    async fn handle(&self, request: &Request) -> Response {
        let visit = Visit::start(&request);

        let response = {
            let mut response = static_assets::handle(request).await;
            if let None = response {
                response = self.shortcuts.handle(request).await;
            }
            if let None = response {
                response = self.blog.handle(request).await;
                info!("Blog handler returned {:?}.", response);
            }
            if let None = response {
                response = crate::visits::handle(&self.visits, request).await;
            }
            match response {
                Some(response) => response,
                None => error_page(404, "Couldn't find the page you're looking for.".into()).await,
            }
        };

        let visit = visit.end(&response);
        self.visits.write().await.register(visit);
        response
    }
}
