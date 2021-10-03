use arangors::{
    document::{
        options::{InsertOptions, RemoveOptions, UpdateOptions},
        response::DocumentResponse,
    },
    AqlQuery, Collection, Database, Document,
};
use serde_json::{Value, to_value};
use std::{
    collections::HashMap,
    convert::Infallible,
    thread,
    vec::Vec,
};
use tokio;
use uclient::reqwest::ReqwestClient;
use warp;

use crate::config::db_database;
use crate::database::{DbConn, DbPool};
use crate::company::{CompanyResponse, FindCompaniesParams};

pub async fn find_companies(
    params: FindCompaniesParams,
    pool: DbPool,
) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn_blocking(move || {
        let conn: DbConn = pool.get().unwrap();
        let db: Database<ReqwestClient> = conn.db(&db_database()).unwrap();

        let mut terms = vec!["FOR x IN companies"];
        let search_term;
        let sort_by_term;
        let limit_term;

        if params.search.is_some() {
            let search: String = params.search.unwrap().trim().to_string().clone();
            if !search.is_empty() {
                search_term = format!("FILTER CONTAINS(x.name, '{}')", search);
                terms.push(search_term.as_str());
            }
        }
        if params.sort_by.is_some() {
            let sort_by: String = params.sort_by.unwrap();
            sort_by_term = format!("SORT x.{} ASC", sort_by);
            terms.push(sort_by_term.as_str());
        }
        if params.limit.is_some() {
            let limit: u32 = params.limit.unwrap();
            limit_term = format!("LIMIT 0, {}", limit);
            terms.push(limit_term.as_str());
        }

        terms.push("RETURN x");
        let q = terms.join(" ");

        let aql = AqlQuery::builder()
            .query(q.as_str())
            .build();
        let records: Vec<CompanyResponse> = db.aql_query(aql).expect("Query failed");
        Ok(warp::reply::json(&records))
    }).await.expect("Task panicked")
}

pub async fn show_company(
    key: String,
    pool: DbPool,
) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn_blocking(move || {
        let conn: DbConn = pool.get().unwrap();
        let db: Database<ReqwestClient> = conn.db(&db_database()).unwrap();
        let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
        let res: Document<CompanyResponse> = collection.document(key.as_ref()).unwrap();
        let record: CompanyResponse = res.document;
        Ok(warp::reply::json(&record))
    }).await.expect("Task panicked")
}
