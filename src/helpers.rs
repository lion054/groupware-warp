use std::result::Result;
use warp::{
    Rejection,
    reply::{Json, WithStatus},
};

pub type JsonResponse = Result<WithStatus<Json>, Rejection>;
