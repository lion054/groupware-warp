use chrono::prelude::*;
use serde::{Deserialize, Serialize};
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
    pub since: Option<NaiveDate>,
}

// update

#[derive(Clone, Debug, Default, Validate, Serialize, Deserialize)]
pub struct UpdateCompanyParams {
    pub name: Option<String>,
    pub since: Option<DateTime<Utc>>,
}

// response

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyResponse {
    pub id: i64,
    pub name: String,
    pub since: NaiveDate,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")] // if none, excluded from query
    pub deleted_at: Option<DateTime<FixedOffset>>,
}

impl CompanyResponse {
    pub fn from_node(node: neo4rs::Node) -> CompanyResponse {
        CompanyResponse {
            id: node.id(),
            name: node.get("name").unwrap(),
            since: node.get("since").unwrap(),
            created_at: node.get("createdAt").unwrap(),
            updated_at: node.get("updatedAt").unwrap(),
            deleted_at: node.get("deletedAt"),
        }
    }
}
