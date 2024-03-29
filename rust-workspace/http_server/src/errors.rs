use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal server error")]
    InternalServerError(#[from] anyhow::Error),
    #[error("Unauthorized request")]
    Unauthorized(String),
}

impl error::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        error!("{self:?}");
        let err_json = json!({ "error": self.to_string() });
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(err_json)
    }
}
