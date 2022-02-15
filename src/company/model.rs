use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

// find

#[derive(Default, Deserialize)]
pub struct FindCompaniesParams {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<String>,
}

lazy_static! {
    static ref REGEX_SORT_BY: Regex = Regex::new(r"(name|capacity)").unwrap();
}

#[derive(Default, Validate)]
pub struct FindCompaniesRequest {
    pub search: Option<String>,
    #[validate(regex = "REGEX_SORT_BY")]
    pub sort_by: Option<String>,
    #[validate(range(min = 5, max = 100))]
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

#[derive(Clone, Debug, Serialize)]
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
    pub fn from_row(row: neo4rs::Row) -> CompanyResponse {
        let c: neo4rs::Node = row.get("c").unwrap();
        CompanyResponse {
            id: c.id(),
            name: c.get("name").unwrap(),
            since: c.get("since").unwrap(),
            created_at: c.get("createdAt").unwrap(),
            updated_at: c.get("updatedAt").unwrap(),
            deleted_at: c.get("deletedAt"),
        }
    }
}
