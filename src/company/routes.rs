use std::{
    collections::HashMap,
    convert::Infallible,
};
use validator::Validate;
use warp::{self, Filter};

use crate::database::DbPool;
use crate::utils::AppError;
use crate::company::{self, FindCompaniesParams};

pub fn init(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_companies(pool.clone())
        .or(show_company(pool.clone()))
        .or(create_company(pool))
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
        .and(warp::body::aggregate())
        .and(with_db(pool))
        .and_then(company::create_company)
}

fn with_db(
    pool: DbPool,
) -> impl Filter<Extract = (DbPool, ), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
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
                    let code = format!("limit: {}", e.to_string());
                    return Err(warp::reject::custom(
                        AppError::ParsingError(code)
                    ));
                },
            };
            params.limit = Some(limit);
        }
        match params.validate() {
            Ok(_) => (),
            Err(e) => {
                return Err(warp::reject::custom(
                    AppError::ValidationError(e)
                ));
            },
        }
        Ok(params)
    })
}
