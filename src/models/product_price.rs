use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    pub price_id: i32,
    pub product_id: i32,
    pub price: f64,
    pub price_type: String,
    pub package_quantity: i32,
    pub remaining_quantity: i32,
    pub created_at: NaiveDateTime,
}

pub async fn get_prices(
    product_id: i32,
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<Price>, Error> {
    let base_query = format!("from product_prices p where p.product_id = {} and p.deleted_at is null", product_id);
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = "p.price_type";

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "p.price_id, p.product_id, p.price::text as price, p.price_type, p.package_quantity, p.remaining_quantity, p.created_at",
        base_query: &base_query,
        search_columns: vec!["p.price_id::varchar", "p.price_type"],
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

    let rows = client
        .query(&result.query, &params_slice)
        .await?;
    let mut prices: Vec<Price> = vec![];
    for row in &rows {
        let price_str: &str = row.get("price");
        prices.push(
            Price {
                price_id: row.get("price_id"),
                product_id: row.get("product_id"),
                price: price_str.parse().unwrap(),
                price_type: row.get("price_type"),
                package_quantity: row.get("package_quantity"),
                remaining_quantity: row.get("remaining_quantity"),
                created_at: row.get("created_at"),
            }
        );
    }

    Ok(PaginationResult {
        data: prices,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct PriceRequest {
    pub product_id: i32,
    pub price: f64,
    pub price_type: String,
    pub package_quantity: i32,
    pub remaining_quantity: i32,
}

pub async fn add_price(
    data: &PriceRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!("insert into product_prices (product_id, price, price_type, package_quantity, remaining_quantity) values ($1, {}, $2, $3, $4)", data.price);
    client
        .execute(
            &query,
            &[&data.product_id, &data.price_type, &data.package_quantity, &data.remaining_quantity],
        )
        .await?;
    Ok(())
}

pub async fn get_price_by_id(price_id: i32, client: &Client) -> Option<Price> {
    let result = client
        .query_one(
            "select p.price_id, p.product_id, p.price::text as price, p.price_type, p.package_quantity, p.remaining_quantity, p.created_at from product_prices p where p.deleted_at is null and p.price_id = $1",
            &[&price_id],
        )
        .await;

    match result {
        Ok(row) =>{ 
            let price_str: &str = row.get("price");
            Some(
                Price {
                    price_id: row.get("price_id"),
                    product_id: row.get("product_id"),
                    price: price_str.parse().unwrap(),
                    price_type: row.get("price_type"),
                    package_quantity: row.get("package_quantity"),
                    remaining_quantity: row.get("remaining_quantity"),
                    created_at: row.get("created_at"),
                })
        },
        Err(_) => None,
    }
}

pub async fn update_price(
    price_id: i32,
    data: &PriceRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!("update product_prices set price = {}, price_type=$1, package_quantity=$2, remaining_quantity=$3 where price_id = $4", data.price);
    client
        .execute(
            &query,
            &[&data.price_type, &data.package_quantity, &data.remaining_quantity, &price_id],
        )
        .await?;

    Ok(())
}

pub async fn delete_price(
    price_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update product_prices set deleted_at = CURRENT_TIMESTAMP where price_id = $1",
            &[&price_id],
        )
        .await?;

    Ok(())
}
