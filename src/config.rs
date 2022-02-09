use std::env;

pub fn host() -> String {
  return env::var("HOST").expect("HOST must be set");
}

pub fn port() -> String {
  return env::var("PORT").expect("PORT must be set");
}

pub fn db_host() -> String {
  return env::var("ARANGODB_HOST").expect("ARANGODB_HOST must be set");
}

pub fn db_port() -> String {
  return env::var("ARANGODB_PORT").expect("ARANGODB_PORT must be set");
}

pub fn db_username() -> String {
  return env::var("ARANGODB_USERNAME").expect("ARANGODB_USERNAME must be set");
}

pub fn db_password() -> String {
  return env::var("ARANGODB_PASSWORD").expect("ARANGODB_PASSWORD must be set");
}

pub fn db_database() -> String {
  return env::var("ARANGODB_DATABASE").expect("ARANGODB_DATABASE must be set");
}
