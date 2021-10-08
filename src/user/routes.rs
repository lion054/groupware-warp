use bytes::{Buf, BufMut};
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    path::Path,
};
use futures::{StreamExt, TryStreamExt};
use serde_json::Deserializer;
use uuid::Uuid;
use validator::Validate;
use warp::{
    multipart::{FormData, Part},
    Filter,
};

use crate::database::DbPool;
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
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_users(pool.clone())
        .or(show_user(pool.clone()))
        .or(create_user(pool.clone()))
        .or(update_user(pool.clone()))
        .or(delete_user(pool))
}

/// GET /users
fn find_users(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::get())
        .and(with_find_request())
        .and(with_db(pool))
        .and_then(user::find_users)
}

/// GET /users/:key
fn show_user(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::get())
        .and(with_db(pool))
        .and_then(user::show_user)
}

/// POST /users
fn create_user(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::post())
        .and(warp::header::exact_ignore_case("content-type", "multipart/form-data"))
        .and(with_create_params())
        .and(with_db(pool))
        .and_then(user::create_user)
}

/// PUT /users/:key
fn update_user(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::put())
        .and(warp::header::exact_ignore_case("content-type", "multipart/form-data"))
        .and(with_update_params())
        .and(with_db(pool))
        .and_then(user::update_user)
}

/// DELETE /users/:key
fn delete_user(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::delete())
        .and(warp::header::exact_ignore_case("content-type", "application/json"))
        .and(with_delete_params())
        .and(with_db(pool))
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
            let sort_by = params.sort_by.unwrap();
            match sort_by.as_str() {
                "name" | "email" => (),
                &_ => {
                    return Err(warp::reject::custom(
                        ApiError::ParsingError("sort_by".to_string(), "Must be one of name and email".to_string())
                    ));
                },
            }
            req.sort_by = Some(sort_by);
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
            if limit < 1 && limit > 100 {
                return Err(warp::reject::custom(
                    ApiError::ParsingError("limit".to_string(), "Must be between 1 and 100".to_string())
                ));
            }
            req.limit = Some(limit);
        }
        Ok(req)
    })
}

fn with_create_params() -> impl Filter<Extract = (CreateUserParams, ), Error = warp::Rejection> + Clone {
    warp::multipart::form().max_length(5_000_000).and_then(validate_create_params)
}

async fn validate_create_params(
    form: FormData,
) -> Result<CreateUserParams, warp::Rejection> {
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
                ApiError::ValidationError(e)
            ))
        },
    }
}

fn with_update_params() -> impl Filter<Extract = (UpdateUserParams, ), Error = warp::Rejection> + Clone {
    warp::multipart::form().max_length(5_000_000).and_then(validate_update_params)
}

async fn validate_update_params(
    form: FormData,
) -> Result<UpdateUserParams, warp::Rejection> {
    println!("123");
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
                ApiError::ValidationError(e)
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
            let mut abs_filepath = env::current_dir().unwrap();
            abs_filepath.push("storage");
            let new_filename = format!("{}.{}", Uuid::new_v4().to_string(), file_extension.unwrap().as_str());
            abs_filepath.push(new_filename.clone());
            tokio::fs::write(&abs_filepath, value).await.map_err(|e| {
                let msg = format!("error writing file: {}", e);
                warp::reject::custom(
                    ApiError::ParsingError("avatar".to_string(), msg)
                )
            }).unwrap();
            let rel_filepath = format!("/storage/{}", new_filename);
            vars.insert(field_name, rel_filepath);
        } else {
            vars.insert(field_name, String::from_utf8(value).unwrap());
        }
    }
    Ok(vars)
}

fn with_delete_params() -> impl Filter<Extract = (DeleteParams, ), Error = warp::Rejection> + Clone {
    warp::body::aggregate().and_then(validate_delete_params)
}

async fn validate_delete_params(
    buf: impl Buf,
) -> Result<DeleteParams, warp::Rejection> {
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
                ApiError::ValidationError(e)
            ));
        },
    }
    Ok(params)
}
