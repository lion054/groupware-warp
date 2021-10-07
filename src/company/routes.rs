use bytes::Buf;
use serde_json::Deserializer;
use std::collections::HashMap;
use validator::Validate;
use warp::Filter;

use crate::database::DbPool;
use crate::helpers::with_db;
use crate::error_handler::ApiError;
use crate::company::{
    self,
    CreateCompanyParams,
    DeleteCompanyParams,
    FindCompaniesParams,
    UpdateCompanyParams,
};

pub fn init(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_companies(pool.clone())
        .or(show_company(pool.clone()))
        .or(create_company(pool.clone()))
        .or(update_company(pool.clone()))
        .or(delete_company(pool))
}

/// GET /companies
fn find_companies(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies")
        .and(warp::get())
        .and(with_find_params())
        .and(with_db(pool))
        .and_then(company::find_companies)
}

/// GET /companies/:key
fn show_company(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies" / String)
        .and(warp::get())
        .and(with_db(pool))
        .and_then(company::show_company)
}

/// POST /companies
fn create_company(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies")
        .and(warp::post())
        .and(with_create_params())
        .and(with_db(pool))
        .and_then(company::create_company)
}

/// PUT /companies/:key
fn update_company(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies" / String)
        .and(warp::put())
        .and(with_update_params())
        .and(with_db(pool))
        .and_then(company::update_company)
}

/// DELETE /companies/:key
fn delete_company(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies" / String)
        .and(warp::delete())
        .and(with_delete_params())
        .and(with_db(pool))
        .and_then(company::delete_company)
}

fn with_find_params() -> impl Filter<Extract = (FindCompaniesParams, ), Error = warp::Rejection> + Clone {
    warp::query::raw().and_then(|qs: String| async move {
        let config = serde_qs::Config::new(0, false);
        let map: HashMap<String, String> = config.deserialize_str(qs.as_str()).unwrap();
        let mut params: FindCompaniesParams = FindCompaniesParams::default();
        if map.contains_key("search") {
            params.search = map.get("search").cloned();
        }
        if map.contains_key("sort_by") {
            params.sort_by = map.get("sort_by").cloned();
        }
        if map.contains_key("limit") {
            let limit = match map.get("limit").unwrap().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    return Err(warp::reject::custom(
                        ApiError::ParsingError("limit".to_string(), e.to_string())
                    ));
                },
            };
            params.limit = Some(limit);
        }
        match params.validate() {
            Ok(_) => (),
            Err(e) => {
                return Err(warp::reject::custom(
                    ApiError::ValidationError(e)
                ));
            },
        }
        Ok(params)
    })
}

fn with_create_params() -> impl Filter<Extract = (CreateCompanyParams, ), Error = warp::Rejection> + Clone {
    warp::body::aggregate().and_then(validate_create_params)
}

async fn validate_create_params(
    buf: impl Buf,
) -> Result<CreateCompanyParams, warp::Rejection> {
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: CreateCompanyParams = match serde_path_to_error::deserialize(deserializer) {
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

fn with_update_params() -> impl Filter<Extract = (UpdateCompanyParams, ), Error = warp::Rejection> + Clone {
    warp::body::aggregate().and_then(validate_update_params)
}

async fn validate_update_params(
    buf: impl Buf,
) -> Result<UpdateCompanyParams, warp::Rejection> {
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: UpdateCompanyParams = match serde_path_to_error::deserialize(deserializer) {
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

fn with_delete_params() -> impl Filter<Extract = (DeleteCompanyParams, ), Error = warp::Rejection> + Clone {
    warp::body::aggregate().and_then(validate_delete_params)
}

async fn validate_delete_params(
    buf: impl Buf,
) -> Result<DeleteCompanyParams, warp::Rejection> {
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: DeleteCompanyParams = match serde_path_to_error::deserialize(deserializer) {
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
