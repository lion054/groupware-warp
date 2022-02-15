use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::{
    convert::Infallible,
    sync::Arc,
};
use validator::Validate;
use warp::Filter;

pub fn with_db(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = (Arc<neo4rs::Graph>, ), Error = Infallible> + Clone {
    warp::any().map(move || {
        graph.clone()
    })
}

// delete

lazy_static! {
    static ref REGEX_THREE_MODES: Regex = Regex::new(r"(erase|trash|restore)").unwrap();
}

#[derive(Debug, Validate, Deserialize)]
pub struct DeleteParams {
    #[validate(regex = "REGEX_THREE_MODES")]
    pub mode: String,
}
