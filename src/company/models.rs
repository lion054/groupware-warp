use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Default, Validate, Deserialize)]
pub struct FindCompaniesParams {
    pub search: Option<String>,
    #[validate(custom = "validate_sort_by")]
    pub sort_by: Option<String>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u32>,
}

fn validate_sort_by(sort_by: &str) -> Result<(), ValidationError> {
    match sort_by {
        "name" | "since" => Ok(()),
        _ => Err(ValidationError::new("Must be one of name and since")),
    }
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
pub struct CreateCompanyParams {
    #[validate(required)]
    pub name: Option<String>,
    #[validate(required)]
    pub since: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCompanyRequest {
    pub name: String,
    pub since: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanyResponse {
    pub _id: String,
    pub _key: String,
    pub _rev: String,
    pub name: String,
    pub since: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub deleted_at: Option<DateTime<Utc>>,
}
