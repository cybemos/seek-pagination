use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Orders {
    pub orders: Vec<Order>,
    pub previous_token: Option<String>,
    pub next_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub creation_date: chrono::DateTime<chrono::Utc>,
    pub update_date: chrono::DateTime<chrono::Utc>,
}

#[derive(FromForm, Debug)]
pub struct OrderSearch {
    pub previous_token: Option<String>,
    pub next_token: Option<String>,
    pub offset: i32,
    pub limit: i32,
}