use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

// find

#[derive(Default, Deserialize)]
pub struct FindCompaniesParams {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<String>,
}

#[derive(Default)]
pub struct FindCompaniesRequest {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<u32>,
}

// create

#[derive(Clone, Debug, Default, Validate, Serialize, Deserialize)]
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

// update

#[derive(Clone, Debug, Default, Validate, Serialize, Deserialize)]
pub struct UpdateCompanyParams {
    pub name: Option<String>,
    pub since: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCompanyRequest {
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub since: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub deleted_at: Option<DateTime<Utc>>,
}

// delete

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrashCompanyRequest {
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub since: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub modified_at: Option<DateTime<Utc>>,
    pub deleted_at: DateTime<Utc>,
}

impl Default for TrashCompanyRequest {
    fn default() -> TrashCompanyRequest {
        TrashCompanyRequest {
            name: None,
            since: None,
            created_at: None,
            modified_at: None,
            deleted_at: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreCompanyRequest {
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub since: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub modified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<Value>, // on response, value will not exist
}

impl Default for RestoreCompanyRequest {
    fn default() -> RestoreCompanyRequest {
        RestoreCompanyRequest {
            name: None,
            since: None,
            created_at: None,
            modified_at: None,
            deleted_at: Some(Value::Null),
        }
    }
}

// response

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
