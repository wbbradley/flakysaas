use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::rt::time::sleep
const SALT: &str = "ab39ncf0-3aldmAS3NK3f";

#[derive(Deserialize)]
struct Request {
    text: String,
}

#[derive(Serialize)]
struct Response {
    text: String,
    hash: String,
    timestamp: u128,
}

async fn handler(req_body: web::Json<Request>) -> impl Responder {
    let mut rng = rand::thread_rng();
    let prob: f64 = rng.gen();

    // Simulate failure 70% of the time
    if prob < 0.7 {
        return HttpResponse::InternalServerError().body("Internal Server Error");
    }

    // Simulate delay 20% of the time
    if prob < 0.9 {
        sleep(Duration::from_secs(200)).await;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let hash_text = format!("{}|{}|{}", SALT, req_body.text, timestamp);
    let mut hasher = Sha256::new();
    hasher.update(hash_text.as_bytes());
    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);

    let resp = Response {
        text: req_body.text.clone(),
        hash: hash_hex,
        timestamp,
    };

    HttpResponse::Ok().json(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(handler)))
        .bind("127.0.0.1:9001")?
        .run()
        .await
}
