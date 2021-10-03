use std::convert::Infallible;
use warp::{self, Filter};

use crate::database::{DbConn, DbPool};
use crate::user;

pub fn init(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_users(pool)
}

/// GET /users
fn find_users(
    pool: DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::get())
        .and(with_db(pool))
        .and_then(user::find_users)
}

fn with_db(
    pool: DbPool,
) -> impl Filter<Extract = (DbPool, ), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn json_body() -> impl Filter<Extract = (user::UserResponse, ), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
