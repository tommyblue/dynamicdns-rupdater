use actix_web::{middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
// use reqwest::json

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    server: bool,

    #[arg(long)]
    client: bool,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 8080)]
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct IpAddress {
    ip_address: String,
}

#[post("/my_ip")]
async fn my_ip(req: HttpRequest) -> impl Responder {
    let addr = req.peer_addr().unwrap();
    let resp = json!({
        "ip_address": addr.ip(),
    });

    HttpResponse::Ok().json(resp)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = Args::parse();

    if !args.server && !args.client {
        log::error!("you must be either server or client");
        std::process::exit(1);
    }

    if args.server && args.client {
        log::error!("cannot be both server and client");
        std::process::exit(1);
    }

    if args.server {
        log::info!(
            "starting HTTP server at http://{0}:{1}",
            args.host,
            args.port
        );
        return HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(web::JsonConfig::default().limit(4096))
                .service(my_ip)
        })
        .bind((args.host, args.port))?
        .run()
        .await;
    }

    if args.client {
        let url = format!("http://{0}:{1}/my_ip", args.host, args.port);
        println!("URL => {url}");
        let client = reqwest::Client::new();
        let res = client
            .post(url)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            .json::<IpAddress>()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        println!("My IP: {}", res.ip_address);
    }

    Ok(())
}
