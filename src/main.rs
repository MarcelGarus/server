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
use std::sync::RwLock;
use std::{convert::Infallible, time::Duration};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

mod shortcuts;
mod static_assets;
mod utils;
mod visits;

const ADDRESS: &'static str = "0.0.0.0:8000";

lazy_static! {
    static ref HANDLER: RootHandler = RootHandler::new();
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

    let make_service = make_service_fn(|_conn| {
        async move {
            let service = service_fn(|request| {
                async move {
                    info!("Original request: {:?}", request);
                    let request = Request::from(&request).unwrap(); // TODO
                    info!("Request: {:?}", request);
                    let response = HANDLER.handle(&request);
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
}
impl RootHandler {
    fn new() -> Self {
        Self {
            visits: RwLock::new(VisitsLog::new()),
            shortcuts: shortcuts::Handler::new(),
        }
    }
    fn handle(&self, request: &Request) -> Response {
        let visit = Visit::start(&request);

        let response = static_assets::handle(request)
            .or_else(|| crate::visits::handle(&self.visits, request))
            .or_else(|| self.shortcuts.handle(request))
            .unwrap_or_else(|| {
                error_page(404, "Couldn't find the page you're looking for.".into())
            });

        let visit = visit.end(&response);
        self.visits.write().unwrap().register(visit);
        response
    }
}
