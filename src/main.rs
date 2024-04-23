use crate::error::*;
use actix_web::rt::time::sleep;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::time::Duration;

use reqwest::Client;

mod error;

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

#[derive(Deserialize)]
struct Params {
    ok: Option<String>,
}

async fn quote_handler(
    req_body: web::Json<Request>,
    query: web::Query<Params>,
) -> Result<impl Responder> {
    log::info!("handling request: {req_body:?}");
    let mut rng = rand::thread_rng();
    let mut prob: f64 = rng.gen();

    if query.ok.is_some() {
        log::info!("sent ok!");
        prob = 1.0;
    }
    // Simulate failure 70% of the time
    if prob < 0.15 {
        sleep(Duration::from_millis((prob * 10_000.0).round() as u64)).await;
        return Err(internal_error("fake error".to_string()));
        // HttpResponse::InternalServerError().body("Internal Server Error");
    }

    // Simulate delay 20% of the time
    if prob < 0.6 {
        log::info!("sleeping for 2s");
        sleep(Duration::from_secs(2)).await;
    }
    if prob < 0.3 {
        log::info!("sleeping for 18s");
        sleep(Duration::from_secs(18)).await;
    }
    let (date, rate) = get_rate_quote(&req_body.quote, &req_body.base).await?;

    let resp = Response {
        date,
        rate,
        quote: req_body.quote.clone().to_uppercase(),
        base: req_body.base.clone().to_uppercase(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

async fn currencies_handler() -> Result<impl Responder> {
    log::info!("handling currencies request");
    let currencies = get_currencies().await?;
    Ok(HttpResponse::Ok().json(currencies))
}

async fn get_currencies() -> Result<Vec<String>> {
    let base = "usd";
    // Simulate delay 20% of the time
    let client = Client::new();
    let json: Value = client
        .get(format!(
            "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/{}.json",
            base.to_lowercase()
        ))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| Error::NotFound)?
        .json::<Value>()
        .await?;
    let Value::Object(map) = json else {
        return Err(internal_error("failed to get object from forex results"));
    };
    let quotes: &Map<String, Value> = map
        .get(base)
        .ok_or(internal_error(format!("could not find {base} in quotes")))?
        .as_object()
        .ok_or(internal_error("faild to find object"))?;
    Ok(quotes
        .keys()
        .map(|value| {
            eprintln!("value = {value:?}");
            value.to_string()
        })
        .collect())
}

async fn get_rate_quote(quote: &str, base: &str) -> Result<(String, f64)> {
    let quote = quote.to_lowercase();
    let base = base.to_lowercase();
    // Simulate delay 20% of the time
    let client = Client::new();
    let json: Value = client
        .get(format!(
            "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/{}.json",
            base.to_lowercase()
        ))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| Error::NotFound)?
        .json::<Value>()
        .await
        .map_err(|_| Error::NotFound)?;
    let Value::Object(map) = json else {
        return Err(internal_error("failed to get object from forex results"));
    };
    let date: &str = map
        .get("date")
        .ok_or(internal_error("could not find date in forex results"))?
        .as_str()
        .unwrap_or("<unknown>");
    let quotes: &Map<String, Value> = map
        .get(&base)
        .ok_or(internal_error(format!("could not find {base} in quotes")))?
        .as_object()
        .ok_or(internal_error(format!(
            "could not get object from {quote} value"
        )))?;
    let rate: f64 = quotes
        .get(&quote)
        .ok_or(Error::NotFound)? // internal_error(format!("could not find {quote} in quotes")))?
        .as_f64()
        .ok_or(internal_error("could not convert rate quote to f64"))?;
    Ok((date.to_string(), rate))
}

#[actix_web::main]
async fn main() {
    let log_filename = "/opt/shared/flakysaas.log";
    eprintln!("logging to {log_filename}...");
    simple_logging::log_to_file(log_filename, log::LevelFilter::Info).unwrap();
    // simple_logging::log_to_stderr(log::LevelFilter::Debug);
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
    HttpServer::new(|| {
        App::new()
            .route("/quote", web::post().to(quote_handler))
            .route("/currencies", web::get().to(currencies_handler))
    })
    .bind("0.0.0.0:9001")
    .unwrap()
    .run()
    .await
    .unwrap();
}
