use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DB(tokio_postgres::Error),
    Pool(deadpool_postgres::PoolError),
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Self {
        Error::DB(error)
    }
}

impl From<deadpool_postgres::PoolError> for Error {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        Error::Pool(error)
    }
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
