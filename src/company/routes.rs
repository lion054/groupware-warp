use std::convert::Infallible;
use warp::{self, Filter};

use crate::database::{DbConn, DbPool};
use crate::company;

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
        .and(warp::query::raw())
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
        .and(warp::body::json())
        .and(with_db(pool))
        .and_then(company::create_company)
}

fn with_db(
    pool: DbPool,
) -> impl Filter<Extract = (DbPool, ), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
