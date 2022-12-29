use chrono::{DateTime, ParseError, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DB(tokio_postgres::Error),
    Pool(deadpool_postgres::PoolError),
    ChonoParse(ParseError),
    Parse(String),
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

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::ChonoParse(error)
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

impl From<&Row> for Order {
    fn from(row: &Row) -> Self {
        return Order {
            id: row.get(0),
            creation_date: row.get(1),
            update_date: row.get(2),
        };
    }
}

#[derive(Debug)]
pub struct OrderSearch {
    pub previous_token: Option<Token>,
    pub next_token: Option<Token>,
    pub offset: i32,
    pub limit: i32,
}

#[derive(Debug)]
pub struct Token {
    pub id: String,
    pub creation_date: DateTime<Utc>,
}

impl From<&Order> for Token {
    fn from(order: &Order) -> Self {
        Token {
            id: order.id.clone(),
            creation_date: order.creation_date,
        }
    }
}

impl Into<String> for Token {
    fn into(self) -> String {
        self.id + &"#" + &self.creation_date.to_rfc3339()
    }
}

impl TryFrom<String> for Token {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let index = value
            .find('#')
            .ok_or(Error::Parse("invalid token : ".to_string() + &value))?;
        Ok(Token {
            id: value[..index].to_string(),
            creation_date: DateTime::parse_from_rfc3339(&value[(index + 1)..])?.into(),
        })
    }
}
