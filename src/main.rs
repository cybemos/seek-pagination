mod repositoy;
mod models;

use rocket::serde::json::Json;
use rocket::State;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use models::{OrderSearch, Order, Orders};
use dotenv::dotenv;
use std::env;

#[macro_use] extern crate rocket;

#[get("/?<previous_token>&<next_token>&<offset>&<limit>")]
async fn get_orders(pool: &State<Pool>, previous_token: Option<String>, next_token: Option<String>, offset: Option<i32>, limit: Option<i32>) -> Json<Orders> {
    let search = OrderSearch {
        previous_token,
        next_token,
        offset: offset.unwrap_or(0),
        limit: limit.unwrap_or(10)
    };
    let mut client = pool.get().await.unwrap();
    let orders = repositoy::get_orders(&mut client, &search).await.unwrap();
    /*let previous_token = orders.get(0).map(|order| order.id.clone());
    let last_index = (search.limit - 1) as usize;
    let next_token = orders.get(last_index).map(|order| order.id.clone());
    Json(Orders { orders, previous_token, next_token })*/
    Json(orders)
}

#[get("/<order_id>")]
async fn get_order_by_id(pool: &State<Pool>, order_id: String) -> Json<Order> {
    let mut client = pool.get().await.unwrap();
    let order = repositoy::get_order_by_id(&mut client, &order_id).await.unwrap();
    Json(order)
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let mut cfg = Config::new();
    cfg.dbname = Some("postgres".to_string());
    cfg.host = env::var("DB_HOST").ok();
    cfg.user = env::var("DB_USER").ok();
    cfg.password = env::var("DB_PASSWORD").ok();
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    rocket::build()
        .manage(pool)
        .mount("/orders", routes![get_orders, get_order_by_id])
}
