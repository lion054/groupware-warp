use arangors::Database;
use std::{
    convert::Infallible,
    result::Result,
};
use uclient::reqwest::ReqwestClient;
use warp::{
    Filter,
    Rejection,
    reply::{Json, WithStatus},
};

use crate::config::db_database;
use crate::database::{DbConn, DbPool};

pub type JsonResult = Result<WithStatus<Json>, Rejection>;

pub fn with_db(
    pool: DbPool,
) -> impl Filter<Extract = (Database<ReqwestClient>, ), Error = Infallible> + Clone {
    warp::any().map(move || {
        let conn: DbConn = pool.get().unwrap();
        conn.db(&db_database()).unwrap()
    })
}
