use std::collections::HashMap;
use validator::Validate;
use warp::Filter;

use crate::database::DbPool;
use crate::helpers::with_db;
use crate::error_handler::ApiError;
use crate::user::{
    self,
    FindUsersParams,
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
        .and(with_find_params())
        .and(with_db(pool))
        .and_then(user::find_users)
}

fn with_find_params() -> impl Filter<Extract = (FindUsersParams, ), Error = warp::Rejection> + Clone {
    warp::query::raw().and_then(|qs: String| async move {
        let config = serde_qs::Config::new(0, false);
        let map: HashMap<String, String> = config.deserialize_str(qs.as_str()).unwrap();
        let mut params: FindUsersParams = FindUsersParams::default();
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
