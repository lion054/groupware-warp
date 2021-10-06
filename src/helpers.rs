use std::result::Result;
use warp::{
    Rejection,
    reply::{Json, WithStatus},
};

pub type JsonResult = Result<WithStatus<Json>, Rejection>;
