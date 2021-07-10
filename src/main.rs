#![feature(async_closure)]

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

mod handlers;
mod shortcuts;
mod utils;
mod visits;

const ADDRESS: &'static str = "0.0.0.0:8000";

lazy_static! {
    static ref HANDLER: Arc<Mutex<handlers::RootHandler>> =
        Arc::new(Mutex::new(handlers::RootHandler::new()));
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

    // let handler = handlers::RootHandler::new();

    // TODO: Maybe find a way to not leak the box.
    // It's not a tragedy that we leak the box here because it's okay to leak one single instance of
    // the handler during the entire lifetime of the program.
    let make_service = make_service_fn(|_conn| {
        async move {
            let service = service_fn(|request| {
                async move {
                    let request = handlers::Request::from(&request).unwrap(); // TODO
                    let response = HANDLER.lock().await.handle(&request);
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
                // .layer(SetResponseHeaderLayer::<_, Request<Body>>::if_not_present(
                //     header::CONTENT_TYPE,
                //     HeaderValue::from_static("application/octet-stream"),
                // ))
                .service(service);
            Ok::<_, Infallible>(service)
        }
    });

    let addr = ADDRESS.parse().unwrap();
    let server = Server::bind(&addr).serve(make_service);

    info!("Listening on http://{}", addr);
    server.await.expect("Error while running the server.");
}
