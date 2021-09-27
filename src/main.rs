#![feature(async_closure)]

use crate::shortcuts::ShortcutDb;
use crate::utils::*;
use crate::visits::{Visit, VisitsLog};
use actix_service::Service;
use actix_web::body::AnyBody;
use actix_web::dev::{self, HttpServiceFactory, RequestHead, ServiceResponse};
use actix_web::http::ContentEncoding;
use actix_web::middleware::{self, ErrorHandlerResponse, ErrorHandlers};
use actix_web::{
    delete, get, guard, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use blog::Blog;
use futures::future::{self, FutureExt};
use http::StatusCode;
use log::{debug, info, warn, LevelFilter};
use rustls::{NoClientAuth, ServerConfig};
use shortcuts::Shortcut;
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use std::io::BufReader;
use std::net::SocketAddr;
use tokio::fs;

mod assets;
mod blog;
mod shortcuts;
mod templates;
mod utils;
mod visits;

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
    let visits_log = web::Data::new(VisitsLog::new().await);
    let blog = web::Data::new(Blog::new().await);
    let shortcut_db = web::Data::new(ShortcutDb::new().await);
    let address = config.address.clone();

    let cloned_log = visits_log.clone();
    let cloned_config = config.clone();
    let server = HttpServer::new(move || {
        let cloned_log = cloned_log.clone();
        let cloned_config = cloned_config.clone();
        let cloned_config2 = cloned_config.clone();
        App::new()
            .app_data(cloned_log.clone())
            .app_data(blog.clone())
            .app_data(shortcut_db.clone())
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
            .service(go_shortcut)
            .service(rss)
            .service(api(&cloned_config2.admin_key))
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
            "schreibe" | "schreib" | "folge" | "folg" => "/about-me",
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
        .append_header(("Cache-Control", "public,max-age=3600"))
        .html(templates::blog_page(blog.list().await.into_iter().rev().collect::<Vec<_>>()).await)
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
                .append_header(("Cache-Control", "public,max-age=3600"))
                .body(content),
            Err(_) => panic!("The file is missing."),
        };
    }

    // Or maybe it's a blog article?
    let blog = req.app_data::<web::Data<Blog>>().unwrap();
    if let Some(article) = blog.get(&key).await {
        return HttpResponse::Ok()
            .append_header(("Cache-Control", "public,max-age=3600"))
            .html(templates::article_page(&article, &blog.get_suggestion(&key).await).await);
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

/// Shortcuts are not content of the website itself. Rather, they redirect to somewhere else.
#[get("/go/{shortcut}")]
async fn go_shortcut(
    req: HttpRequest,
    path: web::Path<(String,)>,
    shortcut_db: web::Data<ShortcutDb>,
) -> impl Responder {
    let (shortcut,) = path.into_inner();
    if let Some(shortcut) = shortcut_db.shortcut_for(&shortcut).await {
        return HttpResponse::redirect_to(&shortcut.url);
    }
    error_page_404(&req).await
}

#[get("/rss")]
async fn rss(blog: web::Data<Blog>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/xml")
        .body(templates::rss_feed(&blog.list().await).await)
}

fn api(admin_key: &str) -> impl HttpServiceFactory {
    web::scope("/api")
        .guard(AuthGuard(admin_key.into()))
        .service(
            web::scope("/shortcuts")
                .service(shortcuts_api::list)
                .service(shortcuts_api::update)
                .service(shortcuts_api::remove),
        )
        .service(web::scope("/blog").service(blog_api::refresh))
        .service(
            web::scope("/visits")
                .service(visits_api::tail)
                .service(visits_api::error_tail)
                .service(visits_api::number_of_visits)
                .service(visits_api::urls)
                .service(visits_api::user_agents)
                .service(visits_api::languages)
                .service(visits_api::referers),
        )
}
pub struct AuthGuard(String);
impl guard::Guard for AuthGuard {
    fn check(&self, req: &RequestHead) -> bool {
        if let Some(val) = req.headers.get("admin-key") {
            return consistenttime::ct_u8_slice_eq(val.as_bytes(), self.0.as_bytes());
        }
        false
    }
}

mod shortcuts_api {
    use super::*;

    #[get("/")]
    async fn list(shortcut_db: web::Data<ShortcutDb>) -> impl Responder {
        let shortcuts = shortcut_db.list().await;
        HttpResponse::Ok().json(shortcuts)
    }

    #[post("/")]
    async fn update(
        shortcut: web::Json<Shortcut>,
        shortcut_db: web::Data<ShortcutDb>,
    ) -> impl Responder {
        shortcut_db.register(shortcut.0).await;
        HttpResponse::Ok().body("Added shortcut.")
    }

    #[delete("/{shortcut}")]
    async fn remove(
        path: web::Path<(String,)>,
        shortcut_db: web::Data<ShortcutDb>,
    ) -> impl Responder {
        let (shortcut,) = path.into_inner();
        shortcut_db.delete(&shortcut).await;
        HttpResponse::Ok().body("Deleted shortcut.")
    }
}

mod blog_api {
    use super::*;

    #[get("/refresh")]
    pub async fn refresh(blog: web::Data<Blog>) -> impl Responder {
        match blog.load().await {
            Ok(_) => HttpResponse::Ok().body("Refreshed blog articles."),
            Err(error) => HttpResponse::InternalServerError().body(error),
        }
    }
}

mod visits_api {
    use super::*;

    #[get("/tail")]
    async fn tail(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_tail().await)
    }

    #[get("/error-tail")]
    async fn error_tail(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_error_tail().await)
    }

    #[get("/number-of-visits")]
    async fn number_of_visits(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_number_of_visits_by_day().await)
    }

    #[get("/urls")]
    async fn urls(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_urls_by_day().await)
    }

    #[get("/user-agents")]
    async fn user_agents(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_user_agents_by_day().await)
    }

    #[get("/languages")]
    async fn languages(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_languages_by_day().await)
    }

    #[get("/referers")]
    async fn referers(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_referers_by_day().await)
    }
}

async fn default_handler(req: HttpRequest) -> impl Responder {
    warn!("Default handler invoked. The request was: {:?}", req);
    error_page_404(&req).await
}

async fn error_page_404(req: &HttpRequest) -> HttpResponse {
    info!("Headers: {:?}", req.headers());
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

async fn error_page(status: StatusCode, title: &str, description: &str) -> HttpResponse {
    HttpResponse::Ok()
        .status(status)
        .html(templates::error_page(status, title, description).await)
}

fn error_500_handler(
    service_response: dev::ServiceResponse<AnyBody>,
) -> actix_web::Result<ErrorHandlerResponse<AnyBody>> {
    let req = service_response.request().clone();
    Ok(ErrorHandlerResponse::Future(Box::pin(async {
        Ok(ServiceResponse::new(
            req,
            error_page(StatusCode::INTERNAL_SERVER_ERROR, "", "").await,
        ))
    })))
}
