use dotenv::dotenv;
use std::sync::Arc;
use warp::{http::Method, Filter};

mod config;
mod database;
mod error_handler;
mod helpers;
mod company;
mod user;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    dotenv().ok();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("Content-Type")
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    let graph: Arc<neo4rs::Graph> = database::init_pool().await;
    let routes = api_filters(graph).with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 7070))
        .await;
}

fn api_filters(
    graph: Arc<neo4rs::Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "v1" / ..) // Add path prefix /api/v1 to all our routes
        .and(
            company::init(graph.clone())
                .or(user::init(graph))
                .recover(error_handler::handle_rejection)
        )
}
