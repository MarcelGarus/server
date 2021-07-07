use log::{info, warn, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tiny_http::{Response, Server};

mod handlers;
mod shortcuts;
mod utils;
mod visits;

const ADDRESS: &'static str = "0.0.0.0:8000";

fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Couldn't initialize logging.");

    let server = Server::http(ADDRESS).unwrap();
    let mut root_handler = handlers::RootHandler::new();
    info!("The server is running at {}.", ADDRESS);

    for tiny_request in server.incoming_requests() {
        let request = match handlers::Request::from_tiny(&tiny_request) {
            Some(request) => request,
            None => continue,
        };
        info!("Incoming request: {:?}", request);
        let response = root_handler.handle(&request);
        let tiny_response =
            Response::from_data(response.body.clone()).with_status_code(response.status_code);
        if let Err(err) = tiny_request.respond(tiny_response) {
            warn!("An error occurred while sending a response: {}", err);
        }
    }
}
