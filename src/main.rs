use actix_web::rt::time::sleep;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
const SALT: &str = "ab39ncf0-3aldmAS3NK3f";

#[derive(Deserialize, Debug)]
struct Request {
    text: String,
}

#[derive(Serialize)]
struct Response {
    text: String,
    hash: String,
    epoch_secs: u64,
}

async fn handler(req_body: web::Json<Request>) -> impl Responder {
    log::info!("handling request: {req_body:?}");
    let mut rng = rand::thread_rng();
    let prob: f64 = rng.gen();

    // Simulate failure 70% of the time
    if prob < 0.7 {
        return HttpResponse::InternalServerError().body("Internal Server Error");
    }

    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // Simulate delay 20% of the time
    if prob < 0.9 {
        sleep(Duration::from_secs(2)).await;
    }

    let hash_text = format!("{}|{}|{}", SALT, req_body.text, epoch_secs);
    let mut hasher = Sha256::new();
    hasher.update(hash_text.as_bytes());
    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);

    let resp = Response {
        text: req_body.text.clone(),
        hash: hash_hex,
        epoch_secs,
    };

    HttpResponse::Ok().json(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logging::log_to_file("/opt/shared/flakysaas.log", log::LevelFilter::Info)?;

    HttpServer::new(|| App::new().route("/", web::post().to(handler)))
        .bind("0.0.0.0:9001")?
        .run()
        .await
}
