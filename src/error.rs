use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use std::panic::Location;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Internal(String, &'static Location<'static>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::NotFound => HttpResponse::NotFound().finish(),
            Error::Internal(message, location) => {
                log::error!(
                    "{}:{}: error occured ({message})",
                    location.file(),
                    location.line()
                );
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    #[track_caller]
    fn from(e: reqwest::Error) -> Self {
        Self::Internal(format!("saw reqwest error: {e:?}"), Location::caller())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Box<dyn std::error::Error>> for Error {
    #[track_caller]
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Internal(format!("dyn error: {error:?}"), Location::caller())
    }
}

impl From<std::io::Error> for Error {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        Self::Internal(format!("io error: {error:?}"), Location::caller())
    }
}

impl From<String> for Error {
    #[track_caller]
    fn from(error: String) -> Self {
        Self::Internal(format!("error: {error:?}"), Location::caller())
    }
}

#[track_caller]
pub fn internal_error(msg: impl Into<String>) -> Error {
    Error::Internal(msg.into(), Location::caller())
}
