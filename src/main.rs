use log::{info, warn, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tiny_http::{Response, Server};
use visits::*;

mod handlers;
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
    let mut visits = VisitsLog::new();
    info!("The server is running at {}.", ADDRESS);

    for request in server.incoming_requests() {
        let visit = Visit::start(&request);
        info!("Incoming request: {:?}", visit);

        let response = handlers::handle(&request);
        let visit = visit.end(&response);

        let response = Response::from_data(response.body).with_status_code(response.status_code);
        if let Err(err) = request.respond(response) {
            warn!("An error occurred while sending a response: {}", err);
        }
        visits.register(visit);
    }
}
