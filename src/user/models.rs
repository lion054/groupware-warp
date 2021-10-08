use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

// find

#[derive(Default, Deserialize)]
pub struct FindUsersParams {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<String>,
}

#[derive(Default)]
pub struct FindUsersRequest {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<u32>,
}

// create

#[derive(Clone, Debug, Default, Validate, Serialize, Deserialize)]
pub struct CreateUserParams {
    #[validate(required)]
    pub name: Option<String>,
    #[validate(required, email)]
    pub email: Option<String>,
    #[validate(required, length(min = 6))]
    pub password: Option<String>,
    #[validate(required, must_match = "password")]
    pub password_confirmation: Option<String>,
    #[validate(required)]
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub avatar: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

// update

#[derive(Clone, Debug, Default, Validate, Serialize, Deserialize)]
pub struct UpdateUserParams {
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 6))]
    pub password: Option<String>,
    #[validate(must_match = "password")]
    pub password_confirmation: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub modified_at: Option<DateTime<Utc>>, // must be None on trash
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub deleted_at: Option<DateTime<Utc>>,
}

// response

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
