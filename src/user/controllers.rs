use arangors::{
    document::{
        options::{InsertOptions, RemoveOptions, UpdateOptions},
        response::DocumentResponse,
    },
    AqlQuery, Collection, Database, Document,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::prelude::*;
use std::{
    convert::Infallible,
    vec::Vec,
};
use uclient::reqwest::ReqwestClient;
use warp::http::StatusCode;

use crate::helpers::{
    DeleteParams,
    JsonResult,
};
use crate::user::{
    CreateUserParams,
    CreateUserRequest,
    FindUsersRequest,
    RestoreUserRequest,
    TrashUserRequest,
    UserResponse,
    UpdateUserParams,
    UpdateUserRequest,
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

pub async fn show_user(
    key: String,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn_blocking(move || {
        let collection: Collection<ReqwestClient> = db.collection("users").unwrap();
        let result: Document<UserResponse> = collection.document(key.as_ref()).unwrap();
        let record: UserResponse = result.document;
        Ok(warp::reply::json(&record))
    }).await.expect("Task panicked")
}

pub async fn create_user(
    params: CreateUserParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let collection: Collection<ReqwestClient> = db.collection("users").unwrap();
    let now = Utc::now();
    let req = CreateUserRequest {
        name: params.name.unwrap(),
        email: params.email.unwrap(),
        password: hash(params.password.unwrap(), DEFAULT_COST).unwrap(),
        avatar: params.avatar.unwrap(),
        created_at: now,
        modified_at: now,
    };
    let options: InsertOptions = InsertOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<CreateUserRequest>> = collection.create_document(Document::new(req), options).unwrap();
    let doc: &CreateUserRequest = res.new_doc().unwrap();
    let record: CreateUserRequest = doc.clone();
    let header = res.header().unwrap();
    let response = UserResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name,
        email: record.email,
        avatar: record.avatar,
        created_at: record.created_at,
        modified_at: record.modified_at,
        deleted_at: None,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::CREATED,
    ))
}

pub async fn update_user(
    key: String,
    params: UpdateUserParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let collection: Collection<ReqwestClient> = db.collection("users").unwrap();
    let req = UpdateUserRequest {
        name: params.name,
        email: params.email,
        password: match params.password {
            Some(pswd) => Some(hash(pswd, DEFAULT_COST).unwrap()),
            None => None,
        },
        avatar: params.avatar,
        created_at: None,
        modified_at: Utc::now(),
        deleted_at: None,
    };
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<UpdateUserRequest>> = collection.update_document(&key, Document::new(req), options).unwrap();
    let doc: &UpdateUserRequest = res.new_doc().unwrap();
    let record: UpdateUserRequest = doc.clone();
    let header = res.header().unwrap();
    let response = UserResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        email: record.email.unwrap(),
        avatar: record.avatar.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at,
        deleted_at: record.deleted_at,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}

pub async fn delete_user(
    key: String,
    params: DeleteParams,
    db: Database<ReqwestClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    tokio::task::spawn_blocking(move || {
        match params.mode.as_str() {
            "erase" => erase_user(key, db),
            "trash" => trash_user(key, db),
            "restore" => restore_user(key, db),
            &_ => {
                let invalid_response: Vec<UserResponse> = vec![];
                Ok(warp::reply::with_status(
                    warp::reply::json(&invalid_response),
                    StatusCode::NO_CONTENT,
                ))
            },
        }
    }).await.expect("Task panicked")
}

fn erase_user(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("users").unwrap();
    let options: RemoveOptions = RemoveOptions::builder()
        .return_old(true)
        .build();

    let res: DocumentResponse<Document<UpdateUserRequest>> = collection.remove_document(&key, options, None).unwrap();
    let doc: &UpdateUserRequest = res.old_doc().unwrap();
    let record: UpdateUserRequest = doc.clone();
    let header = res.header().unwrap();
    let response = UserResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        email: record.email.unwrap(),
        avatar: record.avatar.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at,
        deleted_at: record.deleted_at,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::NO_CONTENT,
    ))
}

fn trash_user(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("users").unwrap();
    let data = TrashUserRequest::default();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<TrashUserRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
    let doc: &TrashUserRequest = res.new_doc().unwrap();
    let record: TrashUserRequest = doc.clone();
    let header = res.header().unwrap();
    let response = UserResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        email: record.email.unwrap(),
        avatar: record.avatar.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: Some(record.deleted_at),
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}

fn restore_user(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("users").unwrap();
    let data = RestoreUserRequest::default();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .keep_null(false)
        .build();

    let res: DocumentResponse<Document<RestoreUserRequest>> = collection.update_document(&key, Document::new(data), options).unwrap();
    let doc: &RestoreUserRequest = res.new_doc().unwrap();
    let record: RestoreUserRequest = doc.clone();
    let header = res.header().unwrap();
    let response = UserResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        email: record.email.unwrap(),
        avatar: record.avatar.unwrap(),
        created_at: record.created_at.unwrap(),
        modified_at: record.modified_at.unwrap(),
        deleted_at: None,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}
