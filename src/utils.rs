use serde::Serialize;
use std::{
    convert::Infallible,
    vec::Vec,
};
use thiserror::Error;
use validator::{ValidationErrors, ValidationErrorsKind};
use warp::{
    cors::CorsForbidden,
    http::StatusCode,
};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    ParsingError(String),
    #[error("validation error: {0}")]
    ValidationError(ValidationErrors),
}

impl warp::reject::Reject for AppError {}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<FieldError>>,
}

#[derive(Serialize)]
struct FieldError {
    name: String,
    errors: Vec<String>,
}

pub async fn handle_rejection(
    r: warp::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    let (
        code,
        message,
        fields,
    ): (
        StatusCode,
        String,
        Option<Vec<FieldError>>,
    ) = if r.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found".to_string(), None)
    } else if let Some(e) = r.find::<CorsForbidden>() {
        (StatusCode::FORBIDDEN, e.to_string(), None)
    } else if let Some(e) = r.find::<AppError>() {
        match e {
            AppError::ParsingError(text) => {
                let pieces: Vec<&str> = text.as_str().split(": ").collect();
                let errors: Vec<FieldError> = vec![FieldError {
                    name: pieces[0].to_string(),
                    errors: vec![pieces[1].to_string()],
                }];
                (StatusCode::BAD_REQUEST, "Parsing errors".to_string(), Some(errors))
            },
            AppError::ValidationError(val_errs) => {
                let errors: Vec<FieldError> = val_errs
                    .errors()
                    .iter()
                    .map(|error_kind| FieldError {
                        name: error_kind.0.to_string(),
                        errors: match error_kind.1 {
                            ValidationErrorsKind::Struct(struct_err) => {
                                validation_errs_to_str_vec(struct_err)
                            },
                            ValidationErrorsKind::Field(field_errs) => field_errs
                                .iter()
                                .map(|fe| format!("{}", fe.code))
                                .collect(),
                            ValidationErrorsKind::List(vec_errs) => vec_errs
                                .iter()
                                .map(|ve| {
                                    let err_text = validation_errs_to_str_vec(ve.1).join(" | ");
                                    format!("{}: {:?}", ve.0, err_text)
                                })
                                .collect(),
                        },
                    })
                    .collect();
                (StatusCode::BAD_REQUEST, "Validation errors".to_string(), Some(errors))
            },
        }
    } else if let Some(e) = r.find::<warp::body::BodyDeserializeError>() {
        (StatusCode::BAD_REQUEST, "Bad request for deserialization".to_string(), None)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), None)
    };

    let json = warp::reply::json(&ErrorResponse {
        success: false,
        message: message.into(),
        fields,
    });

    Ok(warp::reply::with_status(json, code))
}

fn validation_errs_to_str_vec(ve: &ValidationErrors) -> Vec<String> {
    ve.field_errors()
        .iter()
        .map(|fe| {
            format!(
                "{}: errors: {}",
                fe.0,
                fe.1.iter()
                    .map(|ve| format!("{}: {:?}", ve.code, ve.params))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        })
        .collect()
}
