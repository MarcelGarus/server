#![feature(async_closure)]

mod assets;
mod blog;
mod maintenance;
mod templates;
mod utils;
mod visits;

use crate::blog::{canonicalize_topic, Blog};
use crate::maintenance::Maintenance;
use crate::utils::*;
use crate::visits::{Visit, VisitsLog};
use actix_service::Service;
use actix_web::HttpResponseBuilder;
use actix_web::{
    body::AnyBody,
    dev::{self, ServiceResponse},
    get,
    http::ContentEncoding,
    middleware::{self, ErrorHandlerResponse, ErrorHandlers},
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use futures::future::{self, FutureExt};
use http::StatusCode;
use itertools::Itertools;
use log::{debug, info, warn, LevelFilter};
use rustls::{NoClientAuth, ServerConfig};
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use std::{io::BufReader, net::SocketAddr};
use tokio::fs;

#[derive(Clone)]
struct Config {
    address: SocketAddr,
    admin_key: String,
    https_config: Option<HttpsConfig>,
}
#[derive(Clone)]
struct HttpsConfig {
    redirect_from_address: Option<SocketAddr>,
    certificate_chain: String,
    private_key: String,
}
impl Config {
    async fn load() -> Self {
        let config = fs::read_to_string("Config.toml")
            .await
            .unwrap()
            .parse::<toml::Value>()
            .unwrap();
        Self {
            address: config["address"].as_str().unwrap().parse().unwrap(),
            admin_key: config["admin_key"].as_str().unwrap().into(),
            https_config: config
                .get("https")
                .and_then(|it| it.as_table())
                .map(|config| HttpsConfig {
                    redirect_from_address: config
                        .get("redirect_from_address")
                        .map(|address| address.as_str().unwrap().parse().unwrap()),
                    certificate_chain: config["certificate_chain"].as_str().unwrap().into(),
                    private_key: config["private_key"].as_str().unwrap().into(),
                }),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let config = web::Data::new(Config::load().await);
    let maintenance = web::Data::new(Maintenance::new());
    let visits_log = web::Data::new(VisitsLog::load().await);
    let blog = web::Data::new(Blog::new().await);
    let address = config.address.clone();

    let cloned_log = visits_log.clone();
    let cloned_config = config.clone();
    let server = HttpServer::new(move || {
        let cloned_log = cloned_log.clone();
        let cloned_config = cloned_config.clone();
        App::new()
            .app_data(cloned_config.clone())
            .app_data(cloned_log.clone())
            .app_data(blog.clone())
            .app_data(maintenance.clone())
            .wrap_fn(move |req, srv| {
                let log = cloned_log.clone();
                let visit = Visit::for_request(&req);
                srv.call(req).then(async move |res| {
                    log.register(visit.finish(&res)).await;
                    res
                })
            })
            .wrap_fn(move |req, srv| {
                let is_running_locally = cloned_config.clone().https_config.is_none();
                debug!("Normalizing request {:?}", &req);
                srv.call(req).then(async move |res| {
                    res.map(|res| {
                        // Don't normalize the request if we're just running locally.
                        if !is_running_locally {
                            if let Some(location) = normalize_request(res.request()) {
                                return res.into_response(HttpResponse::redirect_to(&location));
                            }
                        }
                        res
                    })
                })
            })
            .wrap(
                ErrorHandlers::new().handler(StatusCode::INTERNAL_SERVER_ERROR, error_500_handler),
            )
            .wrap(middleware::Compress::new(ContentEncoding::Auto))
            .wrap(middleware::NormalizePath::default())
            .service(index)
            .service(pay)
            .service(pay_amount)
            .service(timeline)
            .service(filtered_timeline)
            .service(blog_file)
            .service(rss)
            .service(admin)
            .service(url_with_key)
            .default_service(web::route().to(default_handler))
    });

    let tls_config = config.https_config.clone().map(|config| {
        let mut tls_config = ServerConfig::new(NoClientAuth::new());
        tls_config
            .set_single_cert(
                load_certs(&config.certificate_chain),
                load_private_key(&config.private_key),
            )
            .unwrap();
        tls_config
    });
    let main_server = if let Some(config) = tls_config {
        info!("Binding using HTTPS.");
        server.bind_rustls(address, config)?
    } else {
        warn!("Binding using insecure HTTP.");
        server.bind(address)?
    };

    let redirect_server = config
        .clone()
        .https_config
        .clone()
        .and_then(|config| config.redirect_from_address)
        .map(|addr| {
            HttpServer::new(move || App::new().default_service(web::route().to(redirect_to_https)))
                .bind(addr)
                .expect("Couln't bind to redirect socket.")
        });

    future::join(
        async {
            main_server.run().await.expect("Main server crashed.");
        },
        async {
            if let Some(server) = redirect_server {
                server.run().await.expect("Redirect server crashed.");
            }
        },
    )
    .await;

    info!("Server ended.");
    visits_log
        .flush()
        .await
        .expect("Couldn't flush visits to disk.");

    info!("Ending server executable.");
    Ok(())
}

async fn redirect_to_https(req: HttpRequest) -> impl Responder {
    let location = normalize_request(&req).unwrap_or("https://marcelgarus.dev".into());
    info!("Redirecting to {}", location);
    HttpResponse::redirect_to(&location)
}

/// Returns `None` if the given URI is properly normalized, or `Some(location)`
/// if the user should be redirected to a new location.
fn normalize_request(req: &HttpRequest) -> Option<String> {
    let host = match req.uri().host() {
        Some(host) => host.to_owned(),
        None => req
            .headers()
            .get_utf8("host")
            .unwrap_or("marcelgarus.dev".into()),
    };
    let path = req.path();
    let additional_path_prefix = if host.ends_with(".marcel.jetzt") {
        let subdomain = host[..host.len() - ".marcel.jetzt".len()].to_owned();
        match subdomain.as_ref() {
            "bezahle" | "bezahl" | "zahle" | "zahl" => "/pay",
            "schreibe" | "schreib" | "folge" | "folg" => "/me",
            _ => "",
        }
    } else {
        ""
    };

    let normalized_host = "marcelgarus.dev".to_owned();
    let normalized_path = format!("{}{}", additional_path_prefix, path);
    if host == normalized_host && path == normalized_path {
        None
    } else {
        Some(format!("https://marcelgarus.dev{}", normalized_path))
    }
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = std::fs::File::open(filename).expect("Can't open the certificate file.");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = std::fs::File::open(filename).expect("Can't open the private key file.");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("Can't parse the .pem file.") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!("No keys found in {:?}.", filename);
}

// Visitors of the main page get a list of all articles.
#[get("/")]
async fn index(blog: web::Data<Blog>) -> impl Responder {
    HttpResponse::Ok()
        .cached()
        .html(templates::blog_page(blog.list().await).await)
}

/// For brevity, most URLs consist of a single key.
#[get("/{key}")]
async fn url_with_key(req: HttpRequest, path: web::Path<(String,)>) -> impl Responder {
    let (key,) = path.into_inner();

    // Check if this is one of the static assets or files.
    if let Some(asset) = assets::asset_for(&key) {
        return match fs::read(&asset.path).await {
            Ok(content) => HttpResponse::Ok()
                .content_type(asset.content_type)
                .cached()
                .body(content),
            Err(_) => panic!("The file is missing."),
        };
    }

    // Or maybe it's a blog article?
    let blog = req.app_data::<web::Data<Blog>>().unwrap();
    if let Some(article) = blog.get(&key).await {
        return HttpResponse::Ok().cached().html(
            templates::article_page(&article, &blog.get_suggestion_for(&article).await).await,
        );
    }

    error_page_404(&req).await
}

#[get("/articles")]
async fn timeline(blog: web::Data<Blog>) -> impl Responder {
    HttpResponse::Ok()
        .cached()
        .html(templates::timeline_page(None, &blog.list().await).await)
}

#[get("/articles/{topic}")]
async fn filtered_timeline(path: web::Path<(String,)>, blog: web::Data<Blog>) -> impl Responder {
    let (topic,) = path.into_inner();
    let topic = blog
        .topics()
        .await
        .into_iter()
        .filter(|it| canonicalize_topic(it) == topic)
        .next()
        .unwrap_or("".to_string());

    let articles = blog
        .list()
        .await
        .into_iter()
        .filter(|article| article.matches_topic(&topic))
        .collect_vec();

    HttpResponse::Ok()
        .cached()
        .html(templates::timeline_page(Some(&topic), &articles).await)
}

#[get("/files/{filename}")]
async fn blog_file(req: HttpRequest, path: web::Path<(String,)>) -> impl Responder {
    let (filename,) = path.into_inner();

    if filename
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c))
    {
        match fs::read(&format!("blog/files/{}", filename)).await {
            Ok(content) => {
                return HttpResponse::Ok().cached().body(content);
            }
            Err(_) => {}
        }
    }

    error_page_404(&req).await
}

#[get("/pay")]
async fn pay() -> impl Responder {
    HttpResponse::redirect_to("https://paypal.me/marcelgarus")
}

#[get("/pay/{amount}")]
async fn pay_amount(amount: web::Path<(String,)>) -> impl Responder {
    let (amount,) = amount.into_inner();
    HttpResponse::redirect_to(&format!("https://paypal.me/marcelgarus/{}", amount))
}

#[get("/rss")]
async fn rss(blog: web::Data<Blog>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/xml")
        .body(templates::rss_feed(&blog.list().await).await)
}

#[get("/admin")]
async fn admin(
    req: HttpRequest,
    config: web::Data<Config>,
    maintenance: web::Data<Maintenance>,
    visits_log: web::Data<VisitsLog>,
) -> impl Responder {
    let is_authenticated = if let Some(key) = req.headers().get("admin-key") {
        consistenttime::ct_u8_slice_eq(key.as_bytes(), config.admin_key.as_bytes())
    } else {
        false
    };
    if !is_authenticated {
        warn!("Unauthenticated access attempt to the admin API.");
        return error_page_403().await;
    }

    let mut json = serde_json::Map::new();

    json.insert(
        "server_uptime".to_string(),
        match maintenance.server_uptime().await {
            Ok(uptime) => uptime.into(),
            Err(err) => err.into(),
        },
    );
    json.insert(
        "server_program_uptime".to_string(),
        serde_json::Value::Number(maintenance.server_program_uptime().num_seconds().into()),
    );
    json.insert(
        "log_file_size".to_string(),
        match maintenance.log_size().await {
            Ok(size) => size.into(),
            Err(err) => err.into(),
        },
    );
    json.insert(
        "visits_tail".to_string(),
        serde_json::to_value(visits_log.get_tail().await).unwrap(),
    );
    json.insert(
        "number_of_visits_by_day".to_string(),
        serde_json::to_value(visits_log.get_number_of_visits_by_day().await).unwrap(),
    );

    HttpResponse::Ok().json(json)
}

async fn default_handler(req: HttpRequest) -> impl Responder {
    warn!("Default handler invoked. The request was: {:?}", req);
    error_page_404(&req).await
}

async fn error_page_403() -> HttpResponse {
    error_page(
        StatusCode::FORBIDDEN,
        "Hello, potentially-me!",
        "<p><b>In case you're future me:</b> Add the <code>admin-key</code> to the request headers. I know, easy to forget.</p><p><b>In case you're <i>not</i> me:</b> Please don't do this, bro. Hacking is not appreciated.",
    )
    .await
}

async fn error_page_404(req: &HttpRequest) -> HttpResponse {
    let description = match req.headers().get_utf8("referer") {
        Some(referer) => format!(
            "Looks like you got here by following an invalid link from <code>{}</code> – there's no content here.",
            referer.html_encode(),
        ),
        None => "Sadly, there's no content here. The URL is invalid.".into(),
    };
    error_page(
        StatusCode::NOT_FOUND,
        "Nope-di-nope. Nothing to see here.",
        &description,
    )
    .await
}

fn error_500_handler(
    service_response: dev::ServiceResponse<AnyBody>,
) -> actix_web::Result<ErrorHandlerResponse<AnyBody>> {
    let req = service_response.request().clone();
    Ok(ErrorHandlerResponse::Future(Box::pin(async {
        Ok(ServiceResponse::new(
            req,
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "TODO: Fix server",
                "Ooops. Looks like I didn't do error handling for this condition.",
            )
            .await,
        ))
    })))
}

async fn error_page(status: StatusCode, title: &str, description: &str) -> HttpResponse {
    HttpResponse::Ok()
        .status(status)
        .html(templates::error_page(status, title, description).await)
}

trait HttpResponseBuilderExt {
    fn cached(&mut self) -> &mut Self;
}
impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn cached(&mut self) -> &mut Self {
        self.append_header(("Cache-Control", "public,max-age=3600"))
    }
}
