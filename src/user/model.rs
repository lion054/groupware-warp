use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

// find

#[derive(Default, Deserialize)]
pub struct FindUsersParams {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<String>,
}

lazy_static! {
    static ref REGEX_SORT_BY: Regex = Regex::new(r"(name|email)").unwrap();
}

#[derive(Default, Validate)]
pub struct FindUsersRequest {
    pub search: Option<String>,
    #[validate(regex = "REGEX_SORT_BY")]
    pub sort_by: Option<String>,
    #[validate(range(min = 5, max = 100))]
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

// response

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub deleted_at: Option<DateTime<FixedOffset>>,
}

impl UserResponse {
    pub fn from_row(row: neo4rs::Row) -> UserResponse {
        let u: neo4rs::Node = row.get("u").unwrap();
        UserResponse {
            id: u.id(),
            name: u.get("name").unwrap(),
            email: u.get("email").unwrap(),
            avatar: u.get("avatar").unwrap(),
            created_at: u.get("createdAt").unwrap(),
            updated_at: u.get("updatedAt").unwrap(),
            deleted_at: u.get("deletedAt"),
        }
    }
}
