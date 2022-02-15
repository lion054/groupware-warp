use std::{
    convert::Infallible,
    sync::Arc,
    vec::Vec,
};
use warp::http::StatusCode;

use crate::company::{
    CompanyResponse,
    CreateCompanyParams,
    FindCompaniesRequest,
    UpdateCompanyParams,
};
use crate::helpers::DeleteParams;

pub async fn find_companies(
    req: FindCompaniesRequest,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut terms = vec!["MATCH (c:Company)"];
    let search_term;
    let sort_by_term;
    let limit_term;

    if req.search.is_some() {
        let search: String = req.search.unwrap().trim().to_string().clone();
        if !search.is_empty() {
            search_term = format!("WHERE c.name CONTAINS '{}'", search);
            terms.push(search_term.as_str());
        }
    }
    if req.sort_by.is_some() {
        let sort_by: String = req.sort_by.unwrap();
        sort_by_term = format!("SORT c.{} ASC", sort_by);
        terms.push(sort_by_term.as_str());
    }
    if req.limit.is_some() {
        let limit: u32 = req.limit.unwrap();
        limit_term = format!("SKIP 0 LIMIT {}", limit);
        terms.push(limit_term.as_str());
    }

    terms.push("RETURN c");
    let q = terms.join(" ");

    let mut result: neo4rs::RowStream = graph.execute(neo4rs::query(&q)).await.unwrap();
    let mut records: Vec<CompanyResponse> = vec![];
    while let Ok(Some(row)) = result.next().await {
        records.push(CompanyResponse::from_row(row));
    }
    Ok(warp::reply::json(&records))
}

pub async fn show_company(
    id: String,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, Infallible> {
    let q: neo4rs::Query = neo4rs::query("
        MATCH (c:Company)
        WHERE id(c) = $id
        RETURN c
    ")
    .param("id", id.parse::<i64>().unwrap());

    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let record: CompanyResponse = CompanyResponse::from_row(row);
    Ok(warp::reply::json(&record))
}

pub async fn create_company(
    params: CreateCompanyParams,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let q: neo4rs::Query = neo4rs::query("
        CREATE (c:Company {
            name: $name,
            since: date($since),
            createdAt: datetime(),
            updatedAt: datetime()
        })
        RETURN c
    ")
    .param("name", params.name.unwrap())
    .param("since", params.since.unwrap());

    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let record: CompanyResponse = CompanyResponse::from_row(row);
    Ok(warp::reply::with_status(
        warp::reply::json(&record),
        StatusCode::CREATED,
    ))
}

pub async fn update_company(
    id: String,
    params: UpdateCompanyParams,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut terms = vec!["MATCH (c:Company)"];
    let w = format!("WHERE id(c) = {}", id);
    terms.push(w.as_str());

    let mut data = vec![];
    match params.name {
        Some(x) => {
            data.push(format!("c.name = '{}'", x));
        },
        None => {},
    }
    match params.since {
        Some(x) => {
            data.push(format!("c.since = date('{}')", x.to_rfc3339()));
        },
        None => {},
    }
    let s = format!("SET {}", data.join(", "));
    terms.push(s.as_str());
    terms.push("RETURN c");

    let t = terms.join(" ");
    let q: neo4rs::Query = neo4rs::query(t.as_str());
    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let record: CompanyResponse = CompanyResponse::from_row(row);
    Ok(warp::reply::with_status(
        warp::reply::json(&record),
        StatusCode::OK,
    ))
}

pub async fn delete_company(
    id: String,
    params: DeleteParams,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let empty: Vec<u8> = vec![];

    match params.mode.as_str() {
        "erase" => {
            let q: neo4rs::Query = neo4rs::query("
                MATCH (c:Company)
                WHERE id(c) = $id
                DETACH DELETE c
            ")
            .param("id", id.parse::<i64>().unwrap());

            graph.execute(q).await.unwrap();
            Ok(warp::reply::with_status(
                warp::reply::json(&empty),
                StatusCode::NO_CONTENT,
            ))
        },
        "trash" => {
            let q: neo4rs::Query = neo4rs::query("
                MATCH (c:Company)
                WHERE id(c) = $id
                SET c.deletedAt = datetime()
                RETURN c
            ")
            .param("id", id.parse::<i64>().unwrap());

            let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
            let row: neo4rs::Row = result.next().await.unwrap().unwrap();
            let record: CompanyResponse = CompanyResponse::from_row(row);
            Ok(warp::reply::with_status(
                warp::reply::json(&record),
                StatusCode::OK,
            ))
        },
        "restore" => {
            let q: neo4rs::Query = neo4rs::query("
                MATCH (c:Company)
                WHERE id(c) = $id
                REMOVE c.deletedAt
                RETURN c
            ")
            .param("id", id.parse::<i64>().unwrap());

            let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
            let row: neo4rs::Row = result.next().await.unwrap().unwrap();
            let record: CompanyResponse = CompanyResponse::from_row(row);
            Ok(warp::reply::with_status(
                warp::reply::json(&record),
                StatusCode::OK,
            ))
        },
        &_ => {
            Ok(warp::reply::with_status(
                warp::reply::json(&empty),
                StatusCode::BAD_REQUEST,
            ))
        },
    }
}
