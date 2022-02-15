use std::sync::Arc;
use crate::config::{db_host, db_port, db_username, db_password, db_database};

pub async fn init_pool() -> Arc<neo4rs::Graph> {
    let addr = format!("{}:{}", db_host(), db_port());
    let config = neo4rs::config()
        .uri(&addr)
        .user(&db_username())
        .password(&db_password())
        .db(&db_database())
        .fetch_size(500)
        .max_connections(10)
        .build()
        .unwrap();
    let graph = neo4rs::Graph::connect(config).await.unwrap();
    Arc::new(graph)
}
