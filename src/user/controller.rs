use bcrypt::{DEFAULT_COST, hash, verify};
use path_slash::{PathBufExt, PathExt};
use std::{
    convert::Infallible,
    env,
    path::{Path, PathBuf},
    sync::Arc,
    vec::Vec,
};
use warp::http::StatusCode;

use crate::error_handler::ApiError;
use crate::user::{
    CreateUserParams,
    FindUsersRequest,
    UserResponse,
    UpdateUserParams,
};
use crate::helpers::DeleteParams;

pub async fn find_users(
    req: FindUsersRequest,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, Infallible> {
    let mut terms = vec!["MATCH (u:User)"];
    let search_term;
    let sort_by_term;
    let limit_term;

    if req.search.is_some() {
        let search: String = req.search.unwrap().trim().to_string().clone();
        if !search.is_empty() {
            search_term = format!("WHERE u.name CONTAINS '{0}' OR u.email CONTAINS '{0}'", search);
            terms.push(search_term.as_str());
        }
    }
    if req.sort_by.is_some() {
        let sort_by: String = req.sort_by.unwrap();
        sort_by_term = format!("SORT u.{} ASC", sort_by);
        terms.push(sort_by_term.as_str());
    }

    terms.push("RETURN u");
    if req.limit.is_some() {
        let limit: u32 = req.limit.unwrap();
        limit_term = format!("SKIP 0 LIMIT {}", limit);
        terms.push(limit_term.as_str());
    }
    let q = terms.join(" ");

    let mut result: neo4rs::RowStream = graph.execute(neo4rs::query(&q)).await.unwrap();
    let mut records: Vec<UserResponse> = vec![];
    while let Ok(Some(row)) = result.next().await {
        records.push(UserResponse::from_row(row));
    }
    Ok(warp::reply::json(&records))
}

pub async fn show_user(
    id: String,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, Infallible> {
    let q: neo4rs::Query = neo4rs::query("
        MATCH (u:User)
        WHERE id(u) = $id
        RETURN u
    ")
    .param("id", id.parse::<i64>().unwrap());

    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let record: UserResponse = UserResponse::from_row(row);
    Ok(warp::reply::json(&record))
}

pub async fn create_user(
    params: CreateUserParams,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let q: neo4rs::Query = neo4rs::query("
        CREATE (u:User {
            name: $name,
            email: $email,
            password: $password,
            createdAt: datetime(),
            updatedAt: datetime()
        })
        RETURN u
    ")
    .param("name", params.name.unwrap())
    .param("email", params.email.unwrap())
    .param("password", hash(params.password.unwrap(), DEFAULT_COST).unwrap());

    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let node: neo4rs::Node = row.get("u").unwrap();

    let mut avatar = format!("/storage/{}", params.avatar.clone().unwrap());

    // move file into record directory
    let mut abs_dirpath = env::current_dir().unwrap();
    abs_dirpath.push("storage");
    abs_dirpath.push(node.id().to_string());
    tokio::fs::create_dir_all(abs_dirpath).await.unwrap();
    let org_rel_filepath = PathBuf::from_slash(avatar);
    let org_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), org_rel_filepath.to_str().unwrap());
    avatar = format!("/storage/{}/{}", node.id(), params.avatar.unwrap());
    let new_rel_filepath = PathBuf::from_slash(avatar.clone());
    let new_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), new_rel_filepath.to_str().unwrap());
    tokio::fs::rename(
        PathBuf::from(org_abs_filepath),
        PathBuf::from(new_abs_filepath),
    ).await.map_err(|e| {
        let msg = format!("error moving file: {}", e);
        warp::reject::custom(
            ApiError::ParsingError("avatar".to_string(), msg)
        )
    }).unwrap();

    // update database for avatar path
    let q: neo4rs::Query = neo4rs::query("
        MATCH (u:User)
        WHERE id(u) = $id
        SET u.avatar = $avatar, u.updatedAt = datetime()
        RETURN u
    ")
    .param("id", node.id())
    .param("avatar", avatar);

    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let record: UserResponse = UserResponse::from_row(row);
    Ok(warp::reply::with_status(
        warp::reply::json(&record),
        StatusCode::CREATED,
    ))
}

pub async fn update_user(
    id: String,
    params: UpdateUserParams,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut avatar = None;

    if params.avatar.is_some() {
        // make sure record directory exists
        let mut abs_dirpath = env::current_dir().unwrap();
        abs_dirpath.push("storage");
        abs_dirpath.push(id.clone());
        tokio::fs::create_dir_all(abs_dirpath).await.unwrap();

        // move new image into record directory
        let org_filename = params.avatar.unwrap();
        let org_rel_filepath = PathBuf::from_slash(format!("/storage/{}", org_filename));
        let org_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), org_rel_filepath.to_str().unwrap());
        let rel_filepath = format!("/storage/{}/{}", id.clone(), org_filename);
        let new_rel_filepath = PathBuf::from_slash(rel_filepath.clone());
        let new_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), new_rel_filepath.to_str().unwrap());
        tokio::fs::rename(
            PathBuf::from(org_abs_filepath),
            PathBuf::from(new_abs_filepath),
        ).await.map_err(|e| {
            let msg = format!("error moving file: {}", e);
            warp::reject::custom(
                ApiError::ParsingError("avatar".to_string(), msg)
            )
        }).unwrap();

        // get original file path
        let q: neo4rs::Query = neo4rs::query("
            MATCH (c:Company)
            WHERE id(c) = $id
            RETURN c
        ")
        .param("id", id.parse::<i64>().unwrap());
        let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
        let row: neo4rs::Row = result.next().await.unwrap().unwrap();
        let node: neo4rs::Node = row.get("u").unwrap();

        // delete old image
        let old_avatar: String = node.get("avatar").unwrap();
        let old_rel_filepath = PathBuf::from_slash(old_avatar);
        let old_abs_filepath = format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), old_rel_filepath.to_str().unwrap());
        tokio::fs::remove_file(
            PathBuf::from(old_abs_filepath)
        ).await.unwrap();
        avatar = Some(rel_filepath);
    }

    let mut terms = vec!["MATCH (u:User)"];
    let w = format!("WHERE id(u) = {}", id);
    terms.push(w.as_str());

    let mut data = vec![];
    match params.name {
        Some(x) => {
            data.push(format!("u.name = '{}'", x));
        },
        None => {},
    }
    match params.email {
        Some(x) => {
            data.push(format!("u.email = '{}'", x));
        },
        None => {},
    }
    match params.password {
        Some(x) => {
            data.push(format!("u.password = '{}'", hash(x, DEFAULT_COST).unwrap()));
        },
        None => {},
    }
    match avatar {
        Some(x) => {
            data.push(format!("u.avatar = '{}'", x));
        },
        None => {},
    }
    let s = format!("SET {}", data.join(", "));
    terms.push(s.as_str());
    terms.push("RETURN u");

    let t = terms.join(" ");
    let q: neo4rs::Query = neo4rs::query(t.as_str());
    let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
    let row: neo4rs::Row = result.next().await.unwrap().unwrap();
    let record: UserResponse = UserResponse::from_row(row);
    Ok(warp::reply::with_status(
        warp::reply::json(&record),
        StatusCode::OK,
    ))
}

pub async fn delete_user(
    id: String,
    params: DeleteParams,
    graph: Arc<neo4rs::Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let empty: Vec<u8> = vec![];

    match params.mode.as_str() {
        "erase" => {
            let q: neo4rs::Query = neo4rs::query("
                MATCH (u:User)
                WHERE id(u) = $id
                DETACH DELETE u
            ")
            .param("id", id.parse::<i64>().unwrap());

            // delete record directory including image file
            let mut abs_dirpath = env::current_dir().unwrap();
            abs_dirpath.push("storage");
            abs_dirpath.push(id.clone());
            tokio::fs::remove_dir_all(abs_dirpath).await.unwrap();

            graph.execute(q).await.unwrap();
            Ok(warp::reply::with_status(
                warp::reply::json(&empty),
                StatusCode::NO_CONTENT,
            ))
        },
        "trash" => {
            let q: neo4rs::Query = neo4rs::query("
                MATCH (u:User)
                WHERE id(u) = $id
                SET u.deletedAt = datetime()
                RETURN u
            ")
            .param("id", id.parse::<i64>().unwrap());

            let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
            let row: neo4rs::Row = result.next().await.unwrap().unwrap();
            let record: UserResponse = UserResponse::from_row(row);
            Ok(warp::reply::with_status(
                warp::reply::json(&record),
                StatusCode::OK,
            ))
        },
        "restore" => {
            let q: neo4rs::Query = neo4rs::query("
                MATCH (u:User)
                WHERE id(u) = $id
                REMOVE u.deletedAt
                RETURN u
            ")
            .param("id", id.parse::<i64>().unwrap());

            let mut result: neo4rs::RowStream = graph.execute(q).await.unwrap();
            let row: neo4rs::Row = result.next().await.unwrap().unwrap();
            let record: UserResponse = UserResponse::from_row(row);
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
