use crate::models::{Order, OrderId, OrderSearch, Orders, Result, Token};
use deadpool_postgres::Object as Client;

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
    let orders = match &search.next_token {
        None => match &search.previous_token {
            None => get_orders_without_token(client, search).await?,
            Some(token) => get_orders_by_previous_token(client, token, search).await?,
        },
        Some(token) => get_orders_by_next_token(client, token, search).await?,
    };
    let previous_token = orders.first().map(|order| Token::from(order).into());
    let next_token = match search.limit >= orders.len() as i32 {
        true => orders.last().map(|order| Token::from(order).into()),
        false => None,
    };
    Ok(Orders {
        orders,
        previous_token,
        next_token,
    })
}

async fn get_orders_by_next_token(
    client: &Client,
    token: &Token,
    search: &OrderSearch,
) -> Result<Vec<Order>> {
    let offset = search.offset as i64;
    let limit = search.limit as i64;
    let rows = client
        .query(
            GET_ORDERS_QUERY1,
            &[&offset, &limit, &token.creation_date, &token.id],
        )
        .await?;
    let orders = rows.iter().map(|row| Order::from(row)).collect::<Vec<_>>();
    Ok(orders)
}

async fn get_orders_by_previous_token(
    client: &Client,
    token: &Token,
    search: &OrderSearch,
) -> Result<Vec<Order>> {
    let offset = search.offset as i64;
    let limit = search.limit as i64;
    let rows = client
        .query(
            GET_ORDERS_QUERY2,
            &[&offset, &limit, &token.creation_date, &token.id],
        )
        .await?;
    let orders = rows
        .iter()
        .rev()
        .map(|row| Order::from(row))
        .collect::<Vec<_>>();
    Ok(orders)
}

async fn get_orders_without_token(client: &Client, search: &OrderSearch) -> Result<Vec<Order>> {
    let offset = search.offset as i64;
    let limit = search.limit as i64;
    let rows = client.query(GET_ORDERS_QUERY, &[&offset, &limit]).await?;
    let orders = rows.iter().map(|row| Order::from(row)).collect::<Vec<_>>();
    Ok(orders)
}

pub async fn get_order_by_id(client: &Client, id: &OrderId) -> Result<Order> {
    let row = client.query_one(GET_ORDER_BY_ID_QUERY, &[&id]).await?;
    let order = Order::from(&row);
    Ok(order)
}
