use serde::Serialize;
use std::{
    convert::Infallible,
    vec::Vec,
};
use thiserror::Error;
use validator::ValidationErrors;
use warp::{
    cors::CorsForbidden,
    http::StatusCode,
};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("parsing error: {0}")]
    ParsingError(String),
    #[error("validation error: {0}")]
    ValidationError(ValidationErrors),
}

impl warp::reject::Reject for AppError {}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
    errors: Option<Vec<FieldError>>,
}

#[derive(Serialize)]
struct FieldError {
    field: String,
    field_errors: Vec<String>,
}

pub async fn handle_rejection(
    r: warp::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    if r.is_not_found() {
        Ok(warp::reply::with_status(
            "Not Found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(e) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(e) = r.find::<warp::body::BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            "Bad request".to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Internal server error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
