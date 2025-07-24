#![allow(dead_code)]
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use std::env;
use std::time::Duration;

pub type DbConnMan = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub type DbConn = r2d2::PooledConnection<ConnectionManager<MysqlConnection>>;

pub fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn clone() -> DbConnMan {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<MysqlConnection>::new(&database_url);

    r2d2::Pool::builder()
        .max_size(
            env::var("DB_POOL_MAX_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15), // ⬅️ use a higher default
        )
        .min_idle(Some(
            env::var("DB_POOL_MIN_IDLE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        ))
        .connection_timeout(Duration::from_secs(
            env::var("DB_CONN_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        ))
        .idle_timeout(Some(Duration::from_secs(
            env::var("DB_IDLE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        )))
        .test_on_check_out(true) // ⬅️ ensures connections are valid
        .build(manager)
        .expect("Failed to create DB connection pool")
}
