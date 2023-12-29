mod handlers;
mod models;

use crate::{
    handlers::{debug, init_debug},
    models::Opts,
};
use clap::Parser;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::handlers::parse_request;
use actix_web::{
    web::{self},
    App, HttpServer,
};

use anyhow::Result;
//use growable_bloom_filter::GrowableBloom;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    // Mise en place des logs et leur affichage en temps réel.
    match std::env::var("RUST_LOG") {
        Ok(val) => std::env::set_var("RUST_LOG", val),
        Err(_) => {
            let level = if opts.debug { "debug" } else { "info" };
            std::env::set_var("RUST_LOG", format!("Server={}", level));
        }
    }
    //Initialisation des logs
    env_logger::init();
    log::info!("Starting Server");
    init_debug();

    // Configuration https
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file(opts.key, SslFiletype::PEM)?;
    builder.set_certificate_chain_file(opts.cert)?;

    //let mut gbloom = GrowableBloom::new(0.05, 10);

    // Démarrage du serveur, indication des endpoints.
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(4096))
            .route("/send", web::post().to(parse_request))
            .route("/debug", web::get().to(debug))
    })
    .bind_openssl((opts.address, opts.port), builder)?
    .run()
    .await?)
}
