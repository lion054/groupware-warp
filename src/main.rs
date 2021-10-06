use dotenv::dotenv;
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

    let pool = database::init_pool().expect("Failed to create pool");
    let routes = api_filters(pool).with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

fn api_filters(
    pool: database::DbPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "v1" / ..) // Add path prefix /api/v1 to all our routes
        .and(
            company::init(pool.clone())
                .or(user::init(pool))
                .recover(error_handler::handle_rejection)
        )
}
