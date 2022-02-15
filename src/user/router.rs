use bytes::{Buf, BufMut};
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    path::Path,
    sync::Arc,
};
use futures::{StreamExt, TryStreamExt};
use serde_json::Deserializer;
use uuid::Uuid;
use validator::Validate;
use warp::{
    http::HeaderValue,
    multipart::{FormData, Part},
    Filter,
};

use crate::helpers::{
    DeleteParams,
    with_db,
};
use crate::error_handler::ApiError;
use crate::user::{
    self,
    CreateUserParams,
    FindUsersParams,
    FindUsersRequest,
    UpdateUserParams,
};

pub fn init(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_users(graph.clone())
        .or(show_user(graph.clone()))
        .or(create_user(graph.clone()))
        .or(update_user(graph.clone()))
        .or(delete_user(graph))
}

/// GET /users
fn find_users(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::get())
        .and(with_find_request())
        .and(with_db(graph))
        .and_then(user::find_users)
}

/// GET /users/:id
fn show_user(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::get())
        .and(with_db(graph))
        .and_then(user::show_user)
}

/// POST /users
fn create_user(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::post())
        .and(with_create_params())
        .and(with_db(graph))
        .and_then(user::create_user)
}

/// PATCH /users/:id
fn update_user(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::patch())
        .and(with_update_params())
        .and(with_db(graph))
        .and_then(user::update_user)
}

/// DELETE /users/:id
fn delete_user(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::delete())
        .and(with_delete_params())
        .and(with_db(graph))
        .and_then(user::delete_user)
}

// warp::query::raw can't hook rejection of InvalidQuery for incorrect data type
// so define FindUsersParams that contains string field
// and define FindUsersRequest that contains number field
// then convert FindUsersParams into FindUsersRequest
fn with_find_request() -> impl Filter<Extract = (FindUsersRequest, ), Error = warp::Rejection> + Clone {
    warp::query::<FindUsersParams>().and_then(|params: FindUsersParams| async move {
        let mut req: FindUsersRequest = FindUsersRequest::default();
        if params.search.is_some() {
            req.search = params.search;
        }
        if params.sort_by.is_some() {
            req.sort_by = params.sort_by;
        }
        if params.limit.is_some() {
            let limit = match params.limit.unwrap().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    return Err(warp::reject::custom(
                        ApiError::ParsingError("limit".to_string(), e.to_string())
                    ));
                },
            };
            req.limit = Some(limit);
        }
        match req.validate() {
            Ok(_) => Ok(req),
            Err(e) => Err(warp::reject::custom(
                ApiError::ValidationErrors(e)
            )),
        }
    })
}

fn with_create_params() -> impl Filter<Extract = (CreateUserParams, ), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::value("content-type"))
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(validate_create_params)
}

async fn validate_create_params(
    content_type: HeaderValue,
    form: FormData,
) -> Result<CreateUserParams, warp::Rejection> {
    if content_type.to_str().unwrap().starts_with("multipart/form-data") == false {
        return Err(warp::reject::custom(
            ApiError::ParsingError("content-type".to_string(), "Must be multipart/form-data".to_string())
        ));
    }
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        println!("{:?}", e);
        warp::reject::custom(
            ApiError::ParsingError("sort_by".to_string(), "Must be one of name and email".to_string())
        )
    }).unwrap();

    let vars: HashMap<String, String> = accept_uploading(parts).await.unwrap();

    let params = CreateUserParams {
        name: if vars.contains_key("name") {
            Some(vars.get("name").unwrap().to_string())
        } else {
            None
        },
        email: if vars.contains_key("email") {
            Some(vars.get("email").unwrap().to_string())
        } else {
            None
        },
        password: if vars.contains_key("password") {
            Some(vars.get("password").unwrap().to_string())
        } else {
            None
        },
        password_confirmation: if vars.contains_key("password_confirmation") {
            Some(vars.get("password_confirmation").unwrap().to_string())
        } else {
            None
        },
        avatar: if vars.contains_key("avatar") {
            Some(vars.get("avatar").unwrap().to_string())
        } else {
            None
        },
    };
    match params.validate() {
        Ok(_) => Ok(params),
        Err(e) => {
            Err(warp::reject::custom(
                ApiError::ValidationErrors(e)
            ))
        },
    }
}

fn with_update_params() -> impl Filter<Extract = (UpdateUserParams, ), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::value("content-type"))
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(validate_update_params)
}

async fn validate_update_params(
    content_type: HeaderValue,
    form: FormData,
) -> Result<UpdateUserParams, warp::Rejection> {
    if content_type.to_str().unwrap().starts_with("multipart/form-data") == false {
        return Err(warp::reject::custom(
            ApiError::ParsingError("content-type".to_string(), "Must be multipart/form-data".to_string())
        ));
    }
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        println!("{:?}", e);
        warp::reject::custom(
            ApiError::ParsingError("sort_by".to_string(), "Must be one of name and email".to_string())
        )
    }).unwrap();

    let vars: HashMap<String, String> = accept_uploading(parts).await.unwrap();

    let params = UpdateUserParams {
        name: if vars.contains_key("name") {
            Some(vars.get("name").unwrap().to_string())
        } else {
            None
        },
        email: if vars.contains_key("email") {
            Some(vars.get("email").unwrap().to_string())
        } else {
            None
        },
        password: if vars.contains_key("password") {
            Some(vars.get("password").unwrap().to_string())
        } else {
            None
        },
        password_confirmation: if vars.contains_key("password_confirmation") {
            Some(vars.get("password_confirmation").unwrap().to_string())
        } else {
            None
        },
        avatar: if vars.contains_key("avatar") {
            Some(vars.get("avatar").unwrap().to_string())
        } else {
            None
        },
    };
    match params.validate() {
        Ok(_) => Ok(params),
        Err(e) => {
            Err(warp::reject::custom(
                ApiError::ValidationErrors(e)
            ))
        },
    }
}

async fn accept_uploading(
    parts: Vec<Part>,
) -> Result<HashMap<String, String>, warp::Rejection> {
    let mut vars: HashMap<String, String> = HashMap::new();
    for p in parts {
        let field_name = p.name().clone().to_string();
        let org_filename = p.filename().clone();
        let mut file_extension: Option<String> = None;
        if org_filename.is_some() {
            let content_type = p.content_type().unwrap();
            if content_type.starts_with("image/") {
                file_extension = Some(Path::new(org_filename.unwrap()).extension().and_then(OsStr::to_str).unwrap().to_string());
            } else {
                let msg = format!("invalid file type found: {}", content_type);
                return Err(warp::reject::custom(
                    ApiError::ParsingError("avatar".to_string(), msg)
                ));
            }
        }

        let value = p.stream().try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        }).await.map_err(|e| {
            let msg = format!("reading file error: {}", e);
            warp::reject::custom(
                ApiError::ParsingError("avatar".to_string(), msg)
            )
        }).unwrap();

        if file_extension.is_some() {
            let mut file_path = env::current_dir().unwrap();
            file_path.push("storage");
            let new_filename = format!("{}.{}", Uuid::new_v4().to_string(), file_extension.unwrap().as_str());
            file_path.push(new_filename.clone());
            tokio::fs::write(&file_path, value).await.map_err(|e| {
                let msg = format!("error writing file: {}", e);
                warp::reject::custom(
                    ApiError::ParsingError("avatar".to_string(), msg)
                )
            }).unwrap();
            vars.insert(field_name, new_filename);
        } else {
            vars.insert(field_name, String::from_utf8(value).unwrap());
        }
    }
    Ok(vars)
}

fn with_delete_params() -> impl Filter<Extract = (DeleteParams, ), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::value("content-type"))
        .and(warp::body::aggregate())
        .and_then(validate_delete_params)
}

async fn validate_delete_params(
    content_type: HeaderValue,
    buf: impl Buf,
) -> Result<DeleteParams, warp::Rejection> {
    if content_type.to_str().unwrap().starts_with("application/json") == false {
        return Err(warp::reject::custom(
            ApiError::ParsingError("content-type".to_string(), "Must be application/json".to_string())
        ));
    }
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: DeleteParams = match serde_path_to_error::deserialize(deserializer) {
        Ok(r) => r,
        Err(e) => {
            let pieces: Vec<String> = e.to_string().as_str().split(": ").map(String::from).collect();
            return Err(warp::reject::custom(
                ApiError::ParsingError(pieces[0].clone(), pieces[1].clone())
            ));
        },
    };
    match params.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(warp::reject::custom(
                ApiError::ValidationErrors(e)
            ));
        },
    }
    Ok(params)
}
