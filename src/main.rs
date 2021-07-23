#![feature(async_closure)]

use crate::shortcuts::ShortcutDb;
use crate::utils::*;
use crate::visits::{Visit, VisitsLog};
use actix_service::Service;
use actix_web::dev::{HttpServiceFactory, RequestHead};
use actix_web::{
    delete, get, guard, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use blog::{Blog, FillInArticleStringExt};
use futures::future::FutureExt;
use log::{error, info, LevelFilter};
use rustls::{NoClientAuth, ServerConfig};
use shortcuts::Shortcut;
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use std::fs;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

mod blog;
mod shortcuts;
mod utils;
mod visits;

#[derive(Clone)]
struct Config {
    address: SocketAddr,
    admin_key: String,
    tls_config: Option<TlsConfig>,
}
#[derive(Clone)]
struct TlsConfig {
    certificate: String,
    key: String,
}
impl Config {
    fn load() -> Self {
        let config = std::fs::read_to_string("Config.toml")
            .unwrap()
            .parse::<toml::Value>()
            .unwrap();
        Self {
            address: config["address"].as_str().unwrap().parse().unwrap(),
            admin_key: config["admin_key"].as_str().unwrap().into(),
            tls_config: config
                .get("certificate")
                .and_then(|it| it.as_table())
                .map(|cert_info| TlsConfig {
                    certificate: cert_info["certificate"].as_str().unwrap().into(),
                    key: cert_info["key"].as_str().unwrap().into(),
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

    let config = web::Data::new(Config::load());
    let visits_log = web::Data::new(VisitsLog::new());
    let blog = web::Data::new(Blog::new().await);
    let shortcut_db = web::Data::new(ShortcutDb::new());
    let address = config.address.clone();

    let tls_config = config.tls_config.clone().map(|config| {
        let mut tls_config = ServerConfig::new(NoClientAuth::new());
        tls_config
            .set_single_cert(
                load_certs(&config.certificate),
                load_private_key(&config.key),
            )
            .unwrap();
        tls_config
    });

    // TODO: Enable compression?
    let server = HttpServer::new(move || {
        let config = Arc::new(config.clone());
        let log = Arc::new(visits_log.clone());
        App::new()
            .app_data(visits_log.clone())
            .app_data(blog.clone())
            .app_data(shortcut_db.clone())
            .wrap_fn(move |req, srv| {
                let log = log.clone();
                let visit = Visit::for_request(&req);
                srv.call(req).then(async move |res| {
                    println!("Fn: Hi from response");
                    log.register(visit.finish(&res)).await;
                    res
                })
            })
            // .wrap(middleware::NormalizePath::default())
            .service(index)
            .service(go_shortcut)
            .service(api(&config.admin_key))
            .service(url_with_key)
            .default_service(web::route().to(default_handler))
    });

    let server = if let Some(tls_config) = tls_config {
        server.bind_rustls(address, tls_config)?
    } else {
        server.bind(address)?
    };

    server.run().await?;

    Ok(())
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!(
        "No keys found in {:?} (encrypted keys not supported)",
        filename
    );
}

// Visitors of mgar.us get a list of all articles.
#[get("/")]
async fn index(blog: web::Data<Blog>) -> impl Responder {
    let page_template = std::fs::read("assets/page.html").unwrap().utf8_or_panic();
    let article_template = std::fs::read("assets/article-teaser.html")
        .unwrap()
        .utf8_or_panic();
    let mut articles = blog.list().await;
    articles.sort_by(|a, b| b.published.cmp(&a.published));
    let articles = articles
        .into_iter()
        .map(|article| article_template.fill_in_article(&article))
        .collect::<Vec<_>>();
    let page = page_template.fill_in_content(&itertools::join(articles, "\n"));
    HttpResponse::Ok().body(page)
}

/// For brevity, most URLs consist of a single key.
#[get("/{key}")]
async fn url_with_key(req: HttpRequest, path: web::Path<(String,)>) -> impl Responder {
    let (key,) = path.into_inner();
    info!("Request: {:?}", req);
    info!("Key: {:?}", key);

    // Check if this is one of the static assets.
    let static_assets = vec!["favicon.ico", "icon.png", "prism.css", "prism.js"];
    for asset in static_assets {
        if key == asset {
            // TODO: Make this async
            return match std::fs::read(&format!("assets/{}", asset)) {
                Ok(content) => HttpResponse::Ok().body(content),
                Err(_) => panic!("The file is missing."),
            };
        }
    }

    // Or maybe it's a blog article?
    let blog = req.app_data::<web::Data<Blog>>().unwrap();
    if let Some(article) = blog.article_for(&key).await {
        let page_template = std::fs::read("assets/page.html").unwrap().utf8_or_panic();
        let article_template = std::fs::read("assets/article-full.html")
            .unwrap()
            .utf8_or_panic();
        let article = article_template.fill_in_article(&article);
        let page = page_template.fill_in_content(&article);
        return HttpResponse::Ok().body(page);
    }

    HttpResponse::Ok().body("Unknown key!")
}

/// Shortcuts are not content of the website itself. Rather, they redirect to somewhere else.
#[get("/go/{shortcut}")]
async fn go_shortcut(
    path: web::Path<(String,)>,
    shortcut_db: web::Data<ShortcutDb>,
) -> impl Responder {
    let (shortcut,) = path.into_inner();
    if let Some(shortcut) = shortcut_db.shortcut_for(&shortcut).await {
        return HttpResponse::Found()
            .append_header(("Location", shortcut.url.clone()))
            .body("");
    }

    info!("Shortcut handler, but shortcut not found!");
    HttpResponse::Ok().body("Shortcut")
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
        .service(web::scope("/visits").service(visits_api::tail))
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

mod visits_api {
    use super::*;

    #[get("/tail")]
    async fn tail(visits_log: web::Data<VisitsLog>) -> impl Responder {
        HttpResponse::Ok().json(visits_log.get_tail().await)
    }
}

async fn default_handler(req: HttpRequest) -> impl Responder {
    error!("Default handler invoked.");
    info!("Request: {:?}", req);
    HttpResponse::NotFound().body("Sadly, nothing to see here!")
}
