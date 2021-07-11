#![feature(async_closure)]

use crate::utils::*;
use crate::visits::{Visit, VisitsLog};
use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::sync::Arc;
use std::{convert::Infallible, time::Duration};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

mod blog;
mod shortcuts;
mod static_assets;
mod utils;
mod visits;

const ADDRESS: &'static str = "0.0.0.0:8000";

lazy_static! {
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

    {
        let mut handler = HANDLER.write().await;
        *handler = Some(RootHandler::new().await);
    }

    let make_service = make_service_fn(|_conn| {
        async move {
            let service = service_fn(|request| {
                async move {
                    let request = Request::from(&request).unwrap(); // TODO
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

    let addr = ADDRESS.parse().unwrap();
    let server = Server::bind(&addr).serve(make_service);

    info!("Listening on http://{}", addr);
    server.await.expect("Error while running the server.");
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
