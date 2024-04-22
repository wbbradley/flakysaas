use actix_web::error::ResponseError;
use actix_web::rt::time::sleep;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use std::time::Duration;

use reqwest::Client;

#[derive(Debug)]
enum Error {
    NotFound,
    Internal,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Error::NotFound => HttpResponse::NotFound().finish(),
            Error::Internal => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        log::error!("saw error: {e:?}");
        Self::Internal
    }
}

#[derive(Deserialize, Debug)]
struct Request {
    quote: String,
    base: String,
}

#[derive(Serialize)]
struct Response {
    date: String,
    rate: f64,
    quote: String,
    base: String,
}

async fn quote_handler(req_body: web::Json<Request>) -> Result<impl Responder, Error> {
    log::info!("handling request: {req_body:?}");
    let mut rng = rand::thread_rng();
    let prob: f64 = rng.gen();

    // Simulate failure 70% of the time
    if prob < 0.3 {
        return Err(Error::Internal);
        // HttpResponse::InternalServerError().body("Internal Server Error");
    }

    // Simulate delay 20% of the time
    if prob < 0.6 {
        sleep(Duration::from_secs(2)).await;
    }
    let (date, rate) = get_rate_quote(&req_body.quote, &req_body.base).await?;

    let resp = Response {
        date,
        rate,
        quote: req_body.quote.clone(),
        base: req_body.base.clone(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

async fn get_rate_quote(quote: &str, base: &str) -> Result<(String, f64), Error> {
    // Simulate delay 20% of the time
    let client = Client::new();
    let json: Value = client
        .get(format!("https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/{quote}.json"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| Error::NotFound)?
        .json::<Value>()
        .await?;
    let Value::Object(map) = json else {
        log::error!("failed to get object from forex results");
        return Err(Error::Internal);
    };
    let date: &str = map
        .get("date")
        .ok_or(Error::Internal)?
        .as_str()
        .unwrap_or("<unknown>");
    let quotes: &Map<String, Value> = map
        .get(quote)
        .ok_or(Error::Internal)?
        .as_object()
        .ok_or(Error::Internal)?;
    let rate: f64 = quotes
        .get(base)
        .ok_or(Error::Internal)?
        .as_f64()
        .ok_or(Error::Internal)?;
    Ok((date.to_string(), rate))
}

#[actix_web::main]
async fn main() {
    // simple_logging::log_to_file("/opt/shared/flakysaas.log", log::LevelFilter::Info).unwrap();;
    simple_logging::log_to_stderr(log::LevelFilter::Info);
    let client = Client::new();
    let text = client
        .get("https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/btc.json")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    log::info!("Test query: {text}");
    HttpServer::new(|| App::new().route("/quote", web::post().to(quote_handler)))
        .bind("0.0.0.0:9001")
        .unwrap()
        .run()
        .await
        .unwrap();
}
