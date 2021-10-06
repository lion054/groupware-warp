use arangors::{
    document::{
        options::{InsertOptions, RemoveOptions, UpdateOptions},
        response::DocumentResponse,
    },
    AqlQuery, Collection, Database, Document,
};
use chrono::prelude::*;
use serde_json::Value;
use std::{
    convert::Infallible,
    vec::Vec,
};
use tokio;
use uclient::reqwest::ReqwestClient;
use warp::{
    self,
    http::StatusCode,
};

use crate::helpers::JsonResponse;
use crate::company::{
    CompanyResponse,
    CreateCompanyParams,
    CreateCompanyRequest,
    DeleteCompanyParams,
    FindCompaniesParams,
    UpdateCompanyParams,
    UpdateCompanyRequest,
};

pub async fn find_companies(
    params: FindCompaniesParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    tokio::task::spawn_blocking(move || {
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

        // don't use HashMap for query binding, in order to avoid panick of tokio worker thread
        let aql = AqlQuery::builder()
            .query(q.as_str())
            .build();
        let records: Vec<CompanyResponse> = db.aql_query(aql).expect("Query failed");
        Ok(warp::reply::json(&records))
    }).await.expect("Task panicked")
}

pub async fn show_company(
    key: String,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn_blocking(move || {
        let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
        let result: Document<CompanyResponse> = collection.document(key.as_ref()).unwrap();
        let record: CompanyResponse = result.document;
        Ok(warp::reply::json(&record))
    }).await.expect("Task panicked")
}

pub async fn create_company(
    params: CreateCompanyParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    tokio::task::spawn_blocking(move || {
        let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
        let now = Utc::now();

        let req = CreateCompanyRequest {
            name: params.name.unwrap().clone(),
            since: params.since.unwrap(),
            created_at: now,
            modified_at: now,
        };
        let options: InsertOptions = InsertOptions::builder()
            .return_new(true)
            .build();
        let result: DocumentResponse<Document<CreateCompanyRequest>> = collection.create_document(Document::new(req), options).unwrap();

        let doc: &CreateCompanyRequest = result.new_doc().unwrap();
        let record: CreateCompanyRequest = doc.clone();
        let header = result.header().unwrap();
        let response = CompanyResponse {
            _id: header._id.clone(),
            _key: header._key.clone(),
            _rev: header._rev.clone(),
            name: record.name,
            since: record.since,
            created_at: record.created_at,
            modified_at: record.modified_at,
            deleted_at: None,
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::CREATED,
        ))
    }).await.expect("Task panicked")
}

pub async fn update_company(
    key: String,
    params: UpdateCompanyParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    tokio::task::spawn_blocking(move || {
        let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
        let obj: Value = serde_json::json!({
            "modified_at": Utc::now(),
        });
        let text: String = serde_json::to_string(&obj).unwrap();
        let mut data: UpdateCompanyRequest = serde_json::from_str::<UpdateCompanyRequest>(&text).unwrap();
        if params.name.is_some() {
            data.name = params.name.clone();
        }
        if params.since.is_some() {
            data.since = params.since.clone();
        }
        let options: UpdateOptions = UpdateOptions::builder()
            .return_new(true)
            .return_old(true)
            .build();
        let res: DocumentResponse<Document<UpdateCompanyRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
        let record: &UpdateCompanyRequest = res.new_doc().unwrap();
        let header = res.header().unwrap();
        let response = CompanyResponse {
            _id: header._id.clone(),
            _key: header._key.clone(),
            _rev: header._rev.clone(),
            name: record.name.clone().unwrap(),
            since: record.since.unwrap(),
            created_at: record.created_at.unwrap(),
            modified_at: record.modified_at.unwrap(),
            deleted_at: record.deleted_at,
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::OK,
        ))
    }).await.expect("Task panicked")
}

pub async fn delete_company(
    key: String,
    params: DeleteCompanyParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    tokio::task::spawn_blocking(move || {
        let response: JsonResponse = match params.mode.as_str() {
            "erase" => erase_company(key, db),
            "trash" => trash_company(key, db),
            "restore" => restore_company(key, db),
            &_ => {
                let invalid_response: Vec<CompanyResponse> = vec![];
                Ok(warp::reply::with_status(
                    warp::reply::json(&invalid_response),
                    StatusCode::NO_CONTENT,
                ))
            },
        };
        return response;
    }).await.expect("Task panicked")
}

fn erase_company(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResponse {
    let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
    let options: RemoveOptions = RemoveOptions::builder()
        .return_old(true)
        .build();
    let res: DocumentResponse<Document<UpdateCompanyRequest>> = collection.remove_document(&key, options, None).unwrap();
    let doc: &UpdateCompanyRequest = res.old_doc().unwrap();
    let record: UpdateCompanyRequest = doc.clone();
    let header = res.header().unwrap();
    let response = CompanyResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        since: record.since.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: record.deleted_at,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::NO_CONTENT,
    ))
}

fn trash_company(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResponse {
    let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
    let obj = serde_json::json!({
        "deleted_at": Utc::now(),
    });
    let text = serde_json::to_string(&obj).unwrap();
    let data: UpdateCompanyRequest = serde_json::from_str::<UpdateCompanyRequest>(&text).unwrap();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .return_old(true)
        .build();
    let res: DocumentResponse<Document<UpdateCompanyRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
    let doc: &UpdateCompanyRequest = res.new_doc().unwrap();
    let record: UpdateCompanyRequest = doc.clone();
    let header = res.header().unwrap();
    let response = CompanyResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        since: record.since.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: record.deleted_at,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}

fn restore_company(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResponse {
    let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
    let data: UpdateCompanyRequest = serde_json::from_str::<UpdateCompanyRequest>("{\"deleted_at\":null}").unwrap();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .return_old(true)
        .keep_null(false)
        .build();
    let res: DocumentResponse<Document<UpdateCompanyRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
    let doc: &UpdateCompanyRequest = res.new_doc().unwrap();
    let record: UpdateCompanyRequest = doc.clone();
    let header = res.header().unwrap();
    let response = CompanyResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        since: record.since.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: record.deleted_at,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}
