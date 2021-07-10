#![feature(async_closure)]

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::{convert::Infallible, time::Duration};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

mod handlers;
mod shortcuts;
mod utils;
mod visits;

const ADDRESS: &'static str = "0.0.0.0:8000";

lazy_static! {
    static ref HANDLER: handlers::RootHandler = handlers::RootHandler::new();
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
                    let request = handlers::Request::from(&request).unwrap(); // TODO
                    info!("Request: {:?}", request);
                    let response = HANDLER.handle(&request);
                    let body: Body = response.body.into();
                    let response = Response::builder()
                        .status(response.status_code)
                        .body(body)
                        .unwrap();
                    let result: Result<Response<Body>, Infallible> = Ok(response);
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
