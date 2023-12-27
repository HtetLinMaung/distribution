use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Brand {
    pub brand_id: i32,
    pub brand_name: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_brands(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Brand>, Error> {
    let base_query =
        "from brands b where b.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Sale" {
        "name"
    } else {
        "b.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "b.brand_id, b.brand_name, b.created_at",
        base_query: &base_query,
        search_columns: vec!["b.brand_id::varchar", "b.brand_name"],
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

    let brands: Vec<Brand> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| Brand {
            brand_id: row.get("brand_id"),
            brand_name: row.get("brand_name"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: brands,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct BrandRequest {
    pub brand_name: String,
}

pub async fn add_brand(
    data: &BrandRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into brands (brand_name) values ($1)",
            &[&data.brand_name],
        )
        .await?;
    Ok(())
}

pub async fn get_brand_by_id(brand_id: i32, client: &Client) -> Option<Brand> {
    let result = client
        .query_one(
            "select b.brand_id, b.brand_name, b.created_at from brands b where b.deleted_at is null and b.brand_id = $1",
            &[&brand_id],
        )
        .await;

    match result {
        Ok(row) => Some(Brand {
            brand_id: row.get("brand_id"),
            brand_name: row.get("brand_name"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_brand(
    brand_id: i32,
    data: &BrandRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update brands set brand_name = $1 where brand_id = $2",
            &[&data.brand_name, &brand_id],
        )
        .await?;

    Ok(())
}

pub async fn delete_brand(
    brand_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update brands set deleted_at = CURRENT_TIMESTAMP where brand_id = $1",
            &[&brand_id],
        )
        .await?;

    Ok(())
}
