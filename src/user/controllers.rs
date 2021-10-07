use arangors::{
    document::{
        options::{InsertOptions, RemoveOptions, UpdateOptions},
        response::DocumentResponse,
    },
    AqlQuery, Collection, Database, Document,
};
use serde_json::Value;
use std::{
    convert::Infallible,
    vec::Vec,
};
use uclient::reqwest::ReqwestClient;
use warp;

use crate::user::{
    FindUsersRequest,
    UserResponse,
};

pub async fn find_users(
    req: FindUsersRequest,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn_blocking(move || {
        let mut terms = vec!["FOR x IN users"];
        let search_term;
        let sort_by_term;
        let limit_term;

        if req.search.is_some() {
            let search: String = req.search.unwrap().trim().to_string().clone();
            if !search.is_empty() {
                search_term = format!("FILTER CONTAINS(x.name, '{}') OR CONTAINS(x.email, '{}')", search, search);
                terms.push(search_term.as_str());
            }
        }
        if req.sort_by.is_some() {
            let sort_by: String = req.sort_by.unwrap();
            sort_by_term = format!("SORT x.{} ASC", sort_by);
            terms.push(sort_by_term.as_str());
        }
        if req.limit.is_some() {
            let limit: u32 = req.limit.unwrap();
            limit_term = format!("LIMIT 0, {}", limit);
            terms.push(limit_term.as_str());
        }

        terms.push("RETURN x");
        let q = terms.join(" ");
        let aql = AqlQuery::builder()
            .query(q.as_str())
            .build();
        let records: Vec<UserResponse> = db.aql_query(aql).expect("Query failed");
        Ok(warp::reply::json(&records))
    }).await.expect("Task panicked")
}
