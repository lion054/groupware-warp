use std::convert::Infallible;
use warp::{self, Filter};

use crate::database::{DbConn, DbPool};
use crate::company;

pub fn init(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_companies(pool)
}

/// GET /companies
fn find_companies(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies")
        .and(warp::query::<company::FindCompaniesParams>())
        .and(with_db(pool))
        .and_then(company::find_companies)
}

fn with_db(
    pool: DbPool,
) -> impl Filter<Extract = (DbPool, ), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn json_body() -> impl Filter<Extract = (company::CompanyResponse, ), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
