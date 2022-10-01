mod api;
mod common;
mod configs;
mod posts;
mod root;

use crate::common::app_state::AppState;
use crate::configs::database::connect_db;
use crate::root::controller::not_found;
use actix_files::Files as Fs;
use actix_redis::RedisActor;
use actix_web::{middleware, web, App, HttpServer};
use api::v1::v1_scope;
use listenfd::ListenFd;
use root::router::root_scope;
use serde::{Deserialize, Serialize};
use std::env;
use tera::Tera;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set");
    let server_url = format!("{}:{}", host, port);

    let redis = RedisActor::start(redis_url);
    let db_con = connect_db(db_url)
        .await
        .expect("Should be a db connection!");

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState {
        templates,
        db_con,
        redis,
    };

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "apps/core/static"))
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .default_service(web::route().to(not_found))
            .configure(init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(&server_url)?,
    }
    .workers(8);

    println!("Starting server at {}", server_url);
    server.run().await?;

    Ok(())
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(v1_scope());
    cfg.service(root_scope());
}
