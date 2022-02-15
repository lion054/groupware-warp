use std::env;

pub fn host() -> String {
    return env::var("HOST").expect("HOST must be set");
}

pub fn port() -> String {
    return env::var("PORT").expect("PORT must be set");
}

pub fn db_host() -> String {
    return env::var("NEO4J_HOST").expect("NEO4J_HOST must be set");
}

pub fn db_port() -> String {
    return env::var("NEO4J_PORT").expect("NEO4J_PORT must be set");
}

pub fn db_username() -> String {
    return env::var("NEO4J_USERNAME").expect("NEO4J_USERNAME must be set");
}

pub fn db_password() -> String {
    return env::var("NEO4J_PASSWORD").expect("NEO4J_PASSWORD must be set");
}

pub fn db_database() -> String {
    return env::var("NEO4J_DATABASE").expect("NEO4J_DATABASE must be set");
}
