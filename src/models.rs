use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DB(tokio_postgres::Error),
    Pool(deadpool_postgres::PoolError)
}

pub type OrderId = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Orders {
    pub orders: Vec<Order>,
    pub previous_token: Option<String>,
    pub next_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub creation_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
}

#[derive(FromForm, Debug)]
pub struct OrderSearch {
    pub previous_token: Option<String>,
    pub next_token: Option<String>,
    pub offset: i32,
    pub limit: i32,
}