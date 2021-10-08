use bytes::BufMut;
use std::{
    collections::HashMap,
    env,
};
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use validator::Validate;
use warp::{
    multipart::{FormData, Part},
    Filter,
};

use crate::database::DbPool;
use crate::helpers::with_db;
use crate::error_handler::ApiError;
use crate::user::{
    self,
    CreateUserParams,
    FindUsersParams,
    FindUsersRequest,
};

pub fn init(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_users(pool.clone())
        .or(show_user(pool.clone()))
        .or(create_user(pool))
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
        .and(with_create_params())
        .and(with_db(pool))
        .and_then(user::create_user)
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

    let mut vars: HashMap<String, String> = HashMap::new();
    for p in parts {
        let field_name = p.name().clone().to_string();
        let content_type = p.content_type();
        let mut file_ending: Option<String> = None;
        if p.filename().is_some() {
            match content_type.unwrap() {
                "application/pdf" => {
                    file_ending = Some("pdf".to_string());
                }
                "image/jpeg" => {
                    file_ending = Some("jpg".to_string());
                }
                "image/png" => {
                    file_ending = Some("png".to_string());
                }
                v => {
                    let msg = format!("invalid file type found: {}", v);
                    return Err(warp::reject::custom(
                        ApiError::ParsingError("avatar".to_string(), msg)
                    ));
                }
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

        if file_ending.is_some() {
            let mut filepath = env::current_dir().unwrap();
            filepath.push("storage");
            let file_name = format!("{}.{}", Uuid::new_v4().to_string(), file_ending.unwrap().as_str());
            filepath.push(file_name.clone());
            tokio::fs::write(&filepath, value).await.map_err(|e| {
                let msg = format!("error writing file: {}", e);
                warp::reject::custom(
                    ApiError::ParsingError("avatar".to_string(), msg)
                )
            }).unwrap();
            let val = format!("/storage/{}", file_name);
            vars.insert(field_name, val);
        } else {
            vars.insert(field_name, String::from_utf8(value).unwrap());
        }
    }

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
