use serde::Deserialize;
use std::{
    convert::Infallible,
    result::Result,
};
use validator::{Validate, ValidationError};
use warp::{
    Filter,
    Rejection,
    reply::{Json, WithStatus},
};

use crate::database::DbPool;

pub type JsonResult = Result<WithStatus<Json>, Rejection>;

pub fn with_db(
    pool: DbPool,
) -> impl Filter<Extract = (DbPool, ), Error = Infallible> + Clone {
    warp::any().map(move || {
        pool.clone()
    })
}

// delete

#[derive(Debug, Validate, Deserialize)]
pub struct DeleteParams {
    #[validate(custom = "validate_mode")]
    pub mode: String,
}

fn validate_mode(mode: &str) -> Result<(), ValidationError> {
    match mode {
        "erase" | "trash" | "restore" => Ok(()),
        _ => Err(ValidationError::new("Wrong mode")),
    }
}
