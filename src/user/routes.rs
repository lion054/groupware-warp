use warp::Filter;

use crate::database::DbPool;
use crate::helpers::with_db;
use crate::error_handler::ApiError;
use crate::user::{
    self,
    FindUsersParams,
    FindUsersRequest,
};

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
        .and(with_find_request())
        .and(with_db(pool))
        .and_then(user::find_users)
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
