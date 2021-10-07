use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

// find

#[derive(Default, Validate, Deserialize)]
pub struct FindUsersParams {
    pub search: Option<String>,
    #[validate(custom = "validate_sort_by")]
    pub sort_by: Option<String>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u32>,
}

fn validate_sort_by(sort_by: &str) -> Result<(), ValidationError> {
    match sort_by {
        "name" | "email" => Ok(()),
        _ => Err(ValidationError::new("Must be one of name and email")),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub _id: String,
    pub _key: String,
    pub _rev: String,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub deleted_at: Option<DateTime<Utc>>,
}
