use dotenv::dotenv;
use warp::{self, Filter};

mod config;
mod database;
mod company;
mod user;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    dotenv().ok();

    let pool = database::init_pool().expect("Failed to create pool");
    let routes = api_filters(pool);

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
        )
}
