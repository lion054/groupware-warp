use serde_json::Deserializer;
use std::sync::Arc;
use validator::Validate;
use warp::{
    http::HeaderValue,
    Buf, Filter,
};

use crate::helpers::{
    DeleteParams,
    with_db,
};
use crate::error_handler::ApiError;
use crate::company::{
    self,
    CreateCompanyParams,
    FindCompaniesParams,
    FindCompaniesRequest,
    UpdateCompanyParams,
};

pub fn init(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    find_companies(graph.clone())
        .or(show_company(graph.clone()))
        .or(create_company(graph.clone()))
        .or(update_company(graph.clone()))
        .or(delete_company(graph))
}

/// GET /companies
fn find_companies(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies")
        .and(warp::get())
        .and(with_find_request())
        .and(with_db(graph))
        .and_then(company::find_companies)
}

/// GET /companies/:id
fn show_company(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies" / String)
        .and(warp::get())
        .and(with_db(graph))
        .and_then(company::show_company)
}

/// POST /companies
fn create_company(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies")
        .and(warp::post())
        .and(with_create_params())
        .and(with_db(graph))
        .and_then(company::create_company)
}

/// PATCH /companies/:id
fn update_company(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies" / String)
        .and(warp::patch())
        .and(with_update_params())
        .and(with_db(graph))
        .and_then(company::update_company)
}

/// DELETE /companies/:id
fn delete_company(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("companies" / String)
        .and(warp::delete())
        .and(with_delete_params())
        .and(with_db(graph))
        .and_then(company::delete_company)
}

// warp::query::raw can't hook rejection of InvalidQuery for incorrect data type
// so define FindCompaniesParams that contains string field
// and define FindCompaniesRequest that contains number field
// then convert FindCompaniesParams into FindCompaniesRequest
fn with_find_request() -> impl Filter<Extract = (FindCompaniesRequest, ), Error = warp::Rejection> + Clone {
    warp::query::<FindCompaniesParams>().and_then(|params: FindCompaniesParams| async move {
        let mut req: FindCompaniesRequest = FindCompaniesRequest::default();
        if params.search.is_some() {
            req.search = params.search;
        }
        if params.sort_by.is_some() {
            let sort_by = params.sort_by.unwrap();
            match sort_by.as_str() {
                "name" | "since" => (),
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

fn with_create_params() -> impl Filter<Extract = (CreateCompanyParams, ), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::value("content-type"))
        .and(warp::body::aggregate())
        .and_then(validate_create_params)
}

async fn validate_create_params(
    content_type: HeaderValue,
    buf: impl Buf,
) -> Result<CreateCompanyParams, warp::Rejection> {
    if content_type.to_str().unwrap().starts_with("application/json") == false {
        return Err(warp::reject::custom(
            ApiError::ParsingError("content-type".to_string(), "Must be application/json".to_string())
        ));
    }
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: CreateCompanyParams = match serde_path_to_error::deserialize(deserializer) {
        Ok(r) => r,
        Err(e) => {
            let pieces: Vec<String> = e.to_string().as_str().split(": ").map(String::from).collect();
            return Err(warp::reject::custom(
                ApiError::ParsingError(pieces[0].clone(), pieces[1].clone())
            ));
        },
    };
    match params.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(warp::reject::custom(
                ApiError::ValidationError(e)
            ));
        },
    }
    Ok(params)
}

fn with_update_params() -> impl Filter<Extract = (UpdateCompanyParams, ), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::value("content-type"))
        .and(warp::body::aggregate())
        .and_then(validate_update_params)
}

async fn validate_update_params(
    content_type: HeaderValue,
    buf: impl Buf,
) -> Result<UpdateCompanyParams, warp::Rejection> {
    if content_type.to_str().unwrap().starts_with("application/json") == false {
        return Err(warp::reject::custom(
            ApiError::ParsingError("content-type".to_string(), "Must be application/json".to_string())
        ));
    }
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: UpdateCompanyParams = match serde_path_to_error::deserialize(deserializer) {
        Ok(r) => r,
        Err(e) => {
            let pieces: Vec<String> = e.to_string().as_str().split(": ").map(String::from).collect();
            return Err(warp::reject::custom(
                ApiError::ParsingError(pieces[0].clone(), pieces[1].clone())
            ));
        },
    };
    match params.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(warp::reject::custom(
                ApiError::ValidationError(e)
            ));
        },
    }
    Ok(params)
}

fn with_delete_params() -> impl Filter<Extract = (DeleteParams, ), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::value("content-type"))
        .and(warp::body::aggregate())
        .and_then(validate_delete_params)
}

async fn validate_delete_params(
    content_type: HeaderValue,
    buf: impl Buf,
) -> Result<DeleteParams, warp::Rejection> {
    if content_type.to_str().unwrap().starts_with("application/json") == false {
        return Err(warp::reject::custom(
            ApiError::ParsingError("content-type".to_string(), "Must be application/json".to_string())
        ));
    }
    let deserializer = &mut Deserializer::from_reader(buf.reader());
    let params: DeleteParams = match serde_path_to_error::deserialize(deserializer) {
        Ok(r) => r,
        Err(e) => {
            let pieces: Vec<String> = e.to_string().as_str().split(": ").map(String::from).collect();
            return Err(warp::reject::custom(
                ApiError::ParsingError(pieces[0].clone(), pieces[1].clone())
            ));
        },
    };
    match params.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(warp::reject::custom(
                ApiError::ValidationError(e)
            ));
        },
    }
    Ok(params)
}
