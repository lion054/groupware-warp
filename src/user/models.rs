use chrono::prelude::*;
use serde::{Deserialize, Serialize};

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
