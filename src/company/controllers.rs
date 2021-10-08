use arangors::{
    document::{
        options::{InsertOptions, RemoveOptions, UpdateOptions},
        response::DocumentResponse,
    },
    AqlQuery, Collection, Database, Document,
};
use chrono::prelude::*;
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

use crate::helpers::JsonResult;
use crate::company::{
    CompanyResponse,
    CreateCompanyParams,
    CreateCompanyRequest,
    DeleteCompanyParams,
    FindCompaniesRequest,
    RestoreCompanyRequest,
    TrashCompanyRequest,
    UpdateCompanyParams,
    UpdateCompanyRequest,
};

pub async fn find_companies(
    req: FindCompaniesRequest,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    tokio::task::spawn_blocking(move || {
        let mut terms = vec!["FOR x IN companies"];
        let search_term;
        let sort_by_term;
        let limit_term;

        if req.search.is_some() {
            let search: String = req.search.unwrap().trim().to_string().clone();
            if !search.is_empty() {
                search_term = format!("FILTER CONTAINS(x.name, '{}')", search);
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
        let req = UpdateCompanyRequest {
            name: params.name,
            since: params.since,
            created_at: None,
            modified_at: Some(Utc::now()),
            deleted_at: None,
        };
        let options: UpdateOptions = UpdateOptions::builder()
            .return_new(true)
            .build();

        let res: DocumentResponse<Document<UpdateCompanyRequest>> = collection.update_document(&key, Document::new(req), options).unwrap();
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
        match params.mode.as_str() {
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
        }
    }).await.expect("Task panicked")
}

fn erase_company(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
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
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
    let data = TrashCompanyRequest::default();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<TrashCompanyRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
    let doc: &TrashCompanyRequest = res.new_doc().unwrap();
    let record: TrashCompanyRequest = doc.clone();
    let header = res.header().unwrap();
    let response = CompanyResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        since: record.since.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: Some(record.deleted_at),
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}

fn restore_company(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("companies").unwrap();
    let data = RestoreCompanyRequest::default();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .keep_null(false)
        .build();

    let res: DocumentResponse<Document<RestoreCompanyRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
    let doc: &RestoreCompanyRequest = res.new_doc().unwrap();
    let record: RestoreCompanyRequest = doc.clone();
    let header = res.header().unwrap();
    let response = CompanyResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        since: record.since.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: None,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}
