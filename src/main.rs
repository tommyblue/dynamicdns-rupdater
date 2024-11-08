use actix_web::{middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use clap::Parser;
use dns_lookup::lookup_host;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

    #[arg(long)]
    hostname: String,

    #[arg(long)]
    username: String,

    #[arg(long)]
    password: String,
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
        if args.hostname.is_empty() {
            log::error!("hostname is required");
            std::process::exit(1);
        }

        if args.username.is_empty() {
            log::error!("username is required");
            std::process::exit(1);
        }

        if args.password.is_empty() {
            log::error!("password is required");
            std::process::exit(1);
        }

        let ips: Vec<std::net::IpAddr> = lookup_host(&args.hostname).unwrap();
        let ip = ips.first().unwrap();
        log::info!("IP address of {0} is {1}", args.hostname, ip);

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

        if res.ip_address != ip.to_string() {
            log::info!("IP address does not match, updating...");

            let update_url = format!(
                "http://api.dynu.com/nic/update?myip={0}&username={1}&password={2}",
                res.ip_address, args.username, args.password
            );

            reqwest::get(&update_url)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            log::info!("IP address updated to {0}", res.ip_address);
        }
    }

    Ok(())
}
