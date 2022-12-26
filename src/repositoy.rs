use crate::models::{Error, Order, OrderId, OrderSearch, Orders, Result};
use chrono::{DateTime, Utc};
use deadpool_postgres::Object as Client;
use tokio_postgres::Row;

const GET_ORDERS_QUERY: &str = "SELECT id, creation_date, update_date
FROM \"order\"
order by creation_date desc, id desc
offset $1 limit $2";

const GET_ORDERS_QUERY1: &str = "SELECT id, creation_date, update_date
FROM \"order\"
where (creation_date, id) < ($3, $4)
order by creation_date desc, id desc
offset $1 limit $2";

const GET_ORDERS_QUERY2: &str = "SELECT id, creation_date, update_date
FROM \"order\"
where (creation_date, id) > ($3, $4)
order by creation_date asc, id asc
offset $1 limit $2";

const GET_ORDER_BY_ID_QUERY: &str = "SELECT id, creation_date, update_date
FROM \"order\"
where id = $1";

pub async fn get_orders(client: &Client, search: &OrderSearch) -> Result<Orders> {
    let offset = search.offset as i64;
    let limit = search.limit as i64;
    let orders = match &search.next_token {
        None => match &search.previous_token {
            None => {
                let rows = client
                    .query(GET_ORDERS_QUERY, &[&offset, &limit])
                    .await
                    .map_err(|err| Error::DB(err))?;
                let orders = rows.iter().map(|row| row_to_order(row)).collect::<Vec<_>>();
                orders
            }
            Some(id) => {
                let last_order = get_order_by_id(client, id).await?;
                let rows = client
                    .query(
                        GET_ORDERS_QUERY2,
                        &[&offset, &limit, &last_order.creation_date, &last_order.id],
                    )
                    .await
                    .map_err(|err| Error::DB(err))?;
                let orders = rows
                    .iter()
                    .rev()
                    .map(|row| row_to_order(row))
                    .collect::<Vec<_>>();
                orders
            }
        },
        Some(id) => {
            let last_order = get_order_by_id(client, id).await?;
            let rows = client
                .query(
                    GET_ORDERS_QUERY1,
                    &[&offset, &limit, &last_order.creation_date, &last_order.id],
                )
                .await
                .map_err(|err| Error::DB(err))?;
            let orders = rows.iter().map(|row| row_to_order(row)).collect::<Vec<_>>();
            orders
        }
    };
    let previous_token = orders.first().map(|order| order.id.clone());
    let next_token = match search.limit >= orders.len() as i32 {
        true => orders.last().map(|order| order.id.clone()),
        false => None,
    };
    Ok(Orders {
        orders,
        previous_token,
        next_token,
    })
}

pub async fn get_order_by_id(client: &Client, id: &OrderId) -> Result<Order> {
    let row = client
        .query_one(GET_ORDER_BY_ID_QUERY, &[&id])
        .await
        .map_err(|err| Error::DB(err))?;
    let order = row_to_order(&row);
    Ok(order)
}

fn row_to_order(row: &Row) -> Order {
    let id: String = row.get(0);
    let creation_date: DateTime<Utc> = row.get(1);
    let update_date: DateTime<Utc> = row.get(2);
    return Order {
        id,
        creation_date,
        update_date,
    };
}
