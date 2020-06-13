use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use actix_web::{error, HttpResponse};
use actix_web::http::{StatusCode, header};

#[derive(Debug)]
pub enum WTError {
    InternalError(Box<dyn Error>),
}

impl<T: Error + 'static> From<T> for WTError {
    fn from(error: T) -> Self {
        return WTError::InternalError(Box::new(error));
    }
}

impl Display for WTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return match self {
            WTError::InternalError(_) => write!(f, "An internal error occurred."),
        }
    }
}

impl error::ResponseError for WTError {
    fn status_code(&self) -> StatusCode {
        match *self {
            WTError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        eprintln!("{:?}", self);
        HttpResponse::build(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
