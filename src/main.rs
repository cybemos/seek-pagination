mod models;
mod repository;

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use dotenv::dotenv;
use models::{Error, Order, OrderSearch, Orders, Result, Token};
use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use rocket::State;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use std::{env, path::PathBuf};
use tokio_postgres::NoTls;

#[macro_use]
extern crate rocket;

macro_rules! time_elapsed {
    ($context:literal, $s:stmt) => {
        let timer = Instant::now();
        $s
        println!("time elapsed for {}: {:?}", $context, timer.elapsed());
    };
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("public/index.html")).await.ok()
}

#[get("/<file..>", rank = 2)]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}

#[get("/?<previous_token>&<next_token>&<offset>&<limit>")]
async fn get_orders(
    pool: &State<Pool>,
    previous_token: Option<String>,
    next_token: Option<String>,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<Json<Orders>> {
    let search = OrderSearch {
        previous_token: previous_token
            .map(|t| Token::try_from(t))
            .map_or(Ok(None), |v| v.map(Some))?,
        next_token: next_token
            .map(|t| Token::try_from(t))
            .map_or(Ok(None), |v| v.map(Some))?,
        offset: offset.unwrap_or(0),
        limit: limit.unwrap_or(10),
    };
    time_elapsed!("get_client", let client = pool.get().await?);
    time_elapsed!("repository::get_orders", let orders = repository::get_orders(&client, &search).await?);
    Ok(Json(orders))
}

#[get("/<order_id>")]
async fn get_order_by_id(pool: &State<Pool>, order_id: String) -> Result<Json<Order>> {
    let client = pool.get().await?;
    time_elapsed!("repository::get_order_by_id", let order = repository::get_order_by_id(&client, &order_id).await?);
    Ok(Json(order))
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, content) = match self {
            Error::DB(_) => (Status::InternalServerError, "db error"),
            Error::Pool(_) => (Status::InternalServerError, "pool error"),
            Error::ChonoParse(_) => (Status::BadRequest, "unable to parse"),
            Error::Parse(_) => (Status::BadRequest, "unable to parse"),
        };
        let json = serde_json::json!(content).to_string();
        println!("error: {:?}", self);
        Response::build()
            .header(ContentType::JSON)
            .status(status)
            .sized_body(json.len(), Cursor::new(json))
            .ok()
    }
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let cfg = Config {
        dbname: Some("postgres".to_string()),
        host: env::var("DB_HOST").ok(),
        user: env::var("DB_USER").ok(),
        password: env::var("DB_PASSWORD").ok(),
        manager: Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        }),
        ..Config::new()
    };
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    rocket::build()
        .manage(pool)
        .mount("/orders", routes![get_orders, get_order_by_id])
        .mount("/", routes![index, files])
}
