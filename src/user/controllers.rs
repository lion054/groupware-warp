use arangors::{
    connection::ReqwestClient,
    document::{
        options::{InsertOptions, RemoveOptions, UpdateOptions},
        response::DocumentResponse,
    },
    AqlQuery, Collection, Database, Document,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::prelude::*;
use path_slash::{PathBufExt, PathExt};
use std::{
    convert::Infallible,
    env,
    path::{Path, PathBuf},
    vec::Vec,
};
use warp::http::StatusCode;

use crate::config::db_database;
use crate::database::DbPool;
use crate::helpers::{
    DeleteParams,
    JsonResult,
};
use crate::error_handler::ApiError;
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
    pool: DbPool,
) -> Result<impl warp::Reply, Infallible> {
    let client = pool.get().await.unwrap();
    let db = client.db(&db_database()).await.unwrap();

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
    let records: Vec<UserResponse> = db.aql_query(aql).await.unwrap();
    Ok(warp::reply::json(&records))
}

pub async fn show_user(
    key: String,
    pool: DbPool,
) -> Result<impl warp::Reply, Infallible> {
    let client = pool.get().await.unwrap();
    let db = client.db(&db_database()).await.unwrap();

    let collection: Collection<ReqwestClient> = db.collection("users").await.unwrap();
    let result: Document<UserResponse> = collection.document(key.as_ref()).await.unwrap();
    let record: UserResponse = result.document;
    Ok(warp::reply::json(&record))
}

pub async fn create_user(
    params: CreateUserParams,
    pool: DbPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let client = pool.get().await.unwrap();
    let db = client.db(&db_database()).await.unwrap();

    let collection: Collection<ReqwestClient> = db.collection("users").await.unwrap();
    let now = Utc::now();
    let mut avatar = format!("/storage/{}", params.avatar.clone().unwrap());
    let req = CreateUserRequest {
        name: params.name.unwrap(),
        email: params.email.unwrap(),
        password: hash(params.password.unwrap(), DEFAULT_COST).unwrap(),
        avatar: avatar.clone(),
        created_at: now,
        updated_at: now,
    };
    let options: InsertOptions = InsertOptions::builder().build();

    let res: DocumentResponse<Document<CreateUserRequest>> = collection.create_document(Document::new(req), options).await.unwrap();
    let header = res.header().unwrap();
    let key = header._key.clone();

    // move file into record directory
    let mut abs_dirpath = env::current_dir().unwrap();
    abs_dirpath.push("storage");
    abs_dirpath.push(key.clone());
    tokio::fs::create_dir_all(abs_dirpath).await.unwrap();
    let org_rel_filepath = PathBuf::from_slash(avatar);
    let org_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), org_rel_filepath.to_str().unwrap());
    avatar = format!("/storage/{}/{}", key, params.avatar.unwrap());
    let new_rel_filepath = PathBuf::from_slash(avatar.clone());
    let new_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), new_rel_filepath.to_str().unwrap());
    tokio::fs::rename(
        PathBuf::from(org_abs_filepath),
        PathBuf::from(new_abs_filepath),
    ).await.map_err(|e| {
        let msg = format!("error moving file: {}", e);
        warp::reject::custom(
            ApiError::ParsingError("avatar".to_string(), msg)
        )
    }).unwrap();

    // update database for avatar path
    let req = UpdateUserRequest {
        name: None,
        email: None,
        password: None,
        avatar: Some(avatar.clone()),
        created_at: None,
        updated_at: Utc::now(),
        deleted_at: None,
    };
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<UpdateUserRequest>> = collection.update_document(&key, Document::new(req), options).await.unwrap();
    let doc: &UpdateUserRequest = res.new_doc().unwrap();
    let record: UpdateUserRequest = doc.clone();
    let header = res.header().unwrap();

    let response = UserResponse {
        _id: header._id.clone(),
        _key: header._key.clone(),
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        email: record.email.unwrap(),
        avatar: avatar,
        created_at: record.created_at.unwrap(),
        updated_at: record.updated_at,
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
    pool: DbPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let client = pool.get().await.unwrap();
    let db = client.db(&db_database()).await.unwrap();

    let collection: Collection<ReqwestClient> = db.collection("users").await.unwrap();
    let result: Document<UserResponse> = collection.document(key.as_ref()).await.unwrap();
    let old_record: UserResponse = result.document;
    let mut avatar = None;

    if params.avatar.is_some() {
        // make sure record directory exists
        let mut abs_dirpath = env::current_dir().unwrap();
        abs_dirpath.push("storage");
        abs_dirpath.push(key.clone());
        tokio::fs::create_dir_all(abs_dirpath).await.unwrap();
        // move new image into record directory
        let org_filename = params.avatar.unwrap();
        let org_rel_filepath = PathBuf::from_slash(format!("/storage/{}", org_filename));
        let org_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), org_rel_filepath.to_str().unwrap());
        let rel_filepath = format!("/storage/{}/{}", key.clone(), org_filename);
        let new_rel_filepath = PathBuf::from_slash(rel_filepath.clone());
        let new_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), new_rel_filepath.to_str().unwrap());
        tokio::fs::rename(
            PathBuf::from(org_abs_filepath),
            PathBuf::from(new_abs_filepath),
        ).await.map_err(|e| {
            let msg = format!("error moving file: {}", e);
            warp::reject::custom(
                ApiError::ParsingError("avatar".to_string(), msg)
            )
        }).unwrap();
        // delete old image
        let old_rel_filepath = PathBuf::from_slash(old_record.avatar);
        let old_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), old_rel_filepath.to_str().unwrap());
        tokio::fs::remove_file(
            PathBuf::from(old_abs_filepath)
        ).await.unwrap();
        avatar = Some(rel_filepath);
    }

    let req = UpdateUserRequest {
        name: params.name,
        email: params.email,
        password: match params.password {
            Some(pswd) => Some(hash(pswd, DEFAULT_COST).unwrap()),
            None => None,
        },
        avatar: avatar,
        created_at: None,
        updated_at: Utc::now(),
        deleted_at: None,
    };
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<UpdateUserRequest>> = collection.update_document(&key, Document::new(req), options).await.unwrap();
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
        updated_at: record.updated_at,
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
    pool: DbPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let client = pool.get().await.unwrap();
    let db = client.db(&db_database()).await.unwrap();

    match params.mode.as_str() {
        "erase" => erase_user(key, db).await,
        "trash" => trash_user(key, db).await,
        "restore" => restore_user(key, db).await,
        &_ => {
            let invalid_response: Vec<UserResponse> = vec![];
            Ok(warp::reply::with_status(
                warp::reply::json(&invalid_response),
                StatusCode::NO_CONTENT,
            ))
        },
    }
}

async fn erase_user(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("users").await.unwrap();
    let options: RemoveOptions = RemoveOptions::builder()
        .return_old(true)
        .build();

    let res: DocumentResponse<Document<UpdateUserRequest>> = collection.remove_document(&key, options, None).await.unwrap();
    let doc: &UpdateUserRequest = res.old_doc().unwrap();
    let record: UpdateUserRequest = doc.clone();
    let header = res.header().unwrap();

    // delete record directory including image file
    let mut abs_dirpath = env::current_dir().unwrap();
    abs_dirpath.push("storage");
    abs_dirpath.push(key.clone());
    tokio::fs::remove_dir_all(abs_dirpath).await.unwrap();

    let response = UserResponse {
        _id: header._id.clone(),
        _key: key,
        _rev: header._rev.clone(),
        name: record.name.unwrap(),
        email: record.email.unwrap(),
        avatar: record.avatar.unwrap(),
        created_at: record.created_at.unwrap(),
        updated_at: record.updated_at,
        deleted_at: record.deleted_at,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::NO_CONTENT,
    ))
}

async fn trash_user(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("users").await.unwrap();
    let data = TrashUserRequest::default();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .build();

    let res: DocumentResponse<Document<TrashUserRequest>> = collection.update_document(&key, Document::new(data), options).await.unwrap();
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
        updated_at: record.updated_at.unwrap(),
        deleted_at: Some(record.deleted_at),
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}

async fn restore_user(
    key: String,
    db: Database<ReqwestClient>,
) -> JsonResult { // don't use opaque type to avoid compile error
    let collection: Collection<ReqwestClient> = db.collection("users").await.unwrap();
    let data = RestoreUserRequest::default();
    let options: UpdateOptions = UpdateOptions::builder()
        .return_new(true)
        .keep_null(false)
        .build();

    let res: DocumentResponse<Document<RestoreUserRequest>> = collection.update_document(&key, Document::new(data), options).await.unwrap();
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
        updated_at: record.updated_at.unwrap(),
        deleted_at: None,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}
