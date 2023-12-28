use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Deserialize)]
pub struct OrderRequest {
    pub shop_id: i32,
    pub order_details: Vec<OrderDetailRequest>,
}

#[derive(Deserialize)]
pub struct OrderDetailRequest {
    pub price_id: i32,
    pub quantity: i32,
}

pub async fn add_order(
    data: &OrderRequest,
    user_id: i32,
    client: &mut Client,
) -> Result<i32, Error> {
    let transaction = client.transaction().await?;

    // Check available quantities first
    for order_detail in &data.order_details {
        let row = transaction
            .query_one(
                "SELECT remaining_quantity FROM product_prices WHERE price_id = $1 AND deleted_at IS NULL FOR UPDATE",
                &[&order_detail.price_id],
            )
            .await?;

        let remaining_quantity: i32 = row.get("remaining_quantity");
        if order_detail.quantity > remaining_quantity {
            transaction.rollback().await?;
            return Ok(0);
        }
    }

    // Insert the order
    let row = transaction
        .query_one(
            "INSERT INTO orders (shop_id, user_id, status, total_amount) VALUES ($1, $2, 'Pending', 0.0) RETURNING order_id",
            &[&data.shop_id, &user_id],
        )
        .await?;
    let order_id: i32 = row.get("order_id");

    // Process each order detail
    for order_detail in &data.order_details {
        // Update remaining quantities
        transaction
            .execute(
                "UPDATE product_prices SET remaining_quantity = remaining_quantity - $1 WHERE price_id = $2 AND deleted_at IS NULL",
                &[&order_detail.quantity, &order_detail.price_id],
            )
            .await?;

        // Get discount ID
        let discount_id = match transaction
            .query_one(
                "SELECT discount_id::text FROM product_discounts WHERE price_id = $1 AND deleted_at IS NULL",
                &[&order_detail.price_id],
            )
            .await
        {
            Ok(row) => row.get("discount_id"),
            Err(_) => "null".to_string(),
        };

        // Insert order details
        transaction
            .execute(
                &format!("INSERT INTO order_details (order_id, price_id, quantity, price_at_order, discount_id) VALUES ($1, $2, $3, (SELECT price FROM product_prices WHERE price_id = $4), {discount_id})"),
                &[
                    &order_id,
                    &order_detail.price_id,
                    &order_detail.quantity,
                    &order_detail.price_id,
                ],
            )
            .await?;
    }

    transaction
        .execute(
            "update orders set total_amount = (select coalesce(sum(price_at_order * quantity), 0.0) from order_details where order_id = $1) where order_id = $2",
            &[&order_id, &order_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(order_id)
}

#[derive(Serialize)]
pub struct Order {
    pub order_id: i32,
    pub shop_name: String,
    pub shop_address: String,
    pub shop_latitude: f64,
    pub shop_longitude: f64,
    pub distributor_name: String,
    pub order_date: NaiveDateTime,
    pub status: String,
    pub total_amount: f64,
}

pub async fn get_orders(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    from_date: &Option<NaiveDate>,
    to_date: &Option<NaiveDate>,
    from_amount: &Option<f64>,
    to_amount: &Option<f64>,
    status: &Option<String>,
    user_id: i32,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Order>, Error> {
    let mut base_query =
        "from orders o join shops s on o.shop_id = s.shop_id join users u on u.user_id = o.user_id where o.deleted_at is null and s.deleted_at is null and u.deleted_at is null"
            .to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if role == "Disributor" {
        params.push(Box::new(user_id));
        base_query = format!("{base_query} and o.user_id = ${}", params.len());
    }

    if from_date.is_some() && to_date.is_some() {
        params.push(Box::new(from_date.unwrap()));
        params.push(Box::new(to_date.unwrap()));
        base_query = format!(
            "{base_query} and o.created_at::date between ${} and ${}",
            params.len() - 1,
            params.len()
        );
    }

    if from_amount.is_some() && to_amount.is_some() {
        base_query = format!(
            "{base_query} and o.total_amount between {} and {}",
            from_amount.unwrap(),
            to_amount.unwrap()
        );
    }

    if let Some(s) = status {
        params.push(Box::new(s));
        base_query = format!("{base_query} and o.status = ${}", params.len());
    }

    let order_options = "o.created_at desc".to_string();

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "o.order_id, s.shop_name, s.address shop_address, coalesce(s.latitude::text, '0.0') shop_latitude, coalesce(s.longitude::text, '0.0') shop_longitude, u.full_name distributor_name, o.order_date, o.status, o.total_amount::text",
        base_query: &base_query,
        search_columns: vec![
            "o.order_id::text",
            "s.shop_name",
            "s.address",
            "u.full_name",
            "o.status", 
        ],
        search: search.as_deref(),
        order_options: Some(&order_options),
        page,
        per_page,
    });

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

    let row = client.query_one(&result.count_query, &params_slice).await?;
    let total: i64 = row.get("total");

    let mut page_counts = 0;
    let mut current_page = 0;
    let mut limit = 0;
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        page_counts = (total as f64 / limit as f64).ceil() as usize;
    }

    let orders = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| {
            let shop_latitude: &str = row.get("shop_latitude");
            let shop_latitude: f64 = shop_latitude.parse().unwrap();

            let shop_longitude: &str = row.get("shop_longitude");
            let shop_longitude: f64 = shop_longitude.parse().unwrap();

            let total_amount: &str = row.get("total_amount");
            let total_amount: f64 = total_amount.parse().unwrap();

            Order {
                order_id: row.get("order_id"),
                shop_name: row.get("shop_name"),
                shop_address: row.get("shop_address"),
                shop_latitude,
                shop_longitude,
                distributor_name: row.get("distributor_name"),
                order_date: row.get("order_date"),
                status: row.get("status"),
                total_amount,
            }
        })
        .collect();

    Ok(PaginationResult {
        data: orders,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Serialize)]
pub struct OrderDetail {
    pub order_detail_id: i32,
    pub order_id: i32,
    pub price_id: i32,
    pub product_name: String,
    pub price_type: String,
    pub quantity: i32,
    pub price_at_order: f64,
    pub discount_id: i32,
    pub discount_name: String,
}

pub async fn get_order_details(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    order_id: Option<i32>,
    from_amount: &Option<f64>,
    to_amount: &Option<f64>,
    client: &Client,
) -> Result<PaginationResult<OrderDetail>, Error> {
    let mut base_query =
        "from order_details od join orders o on o.order_id = od.order_id join product_prices pp on pp.price_id = od.price_id join products p on p.product_id = pp.product_id left join discounts d on d.discount_id = od.discount_id where o.deleted_at is null and od.deleted_at is null and pp.deleted_at is null and p.deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if let Some(oid) = order_id {
        params.push(Box::new(oid));
        base_query = format!("{base_query} and od.order_id = ${}", params.len());
    }

    if from_amount.is_some() && to_amount.is_some() {
        base_query = format!(
            "{base_query} and od.price_at_order between {} and {}",
            from_amount.unwrap(),
            to_amount.unwrap()
        );
    }

    let order_options = "p.product_name".to_string();

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "od.order_detail_id, o.order_id, od.price_id, p.product_name, pp.price_type, od.quantity, od.price_at_order::text, coalesce(od.discount_id, 0) discount_id, coalesce(d.discount_name, '') discount_name",
        base_query: &base_query,
        search_columns: vec![
            "od.order_detail_id::text",
            "o.order_id::text",
            "p.product_name",
            "pp.price_type",
        ],
        search: search.as_deref(),
        order_options: Some(&order_options),
        page,
        per_page,
    });

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

    let row = client.query_one(&result.count_query, &params_slice).await?;
    let total: i64 = row.get("total");

    let mut page_counts = 0;
    let mut current_page = 0;
    let mut limit = 0;
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        page_counts = (total as f64 / limit as f64).ceil() as usize;
    }

    let order_detailss = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| {
            let price_at_order: &str = row.get("price_at_order");
            let price_at_order: f64 = price_at_order.parse().unwrap();

            OrderDetail {
                order_detail_id: row.get("order_detail_id"),
                order_id: row.get("order_id"),
                price_id: row.get("price_id"),
                product_name: row.get("product_name"),
                price_type: row.get("price_type"),
                quantity: row.get("quantity"),
                price_at_order,
                discount_id: row.get("discount_id"),
                discount_name: row.get("discount_name"),
            }
        })
        .collect();

    Ok(PaginationResult {
        data: order_detailss,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}
