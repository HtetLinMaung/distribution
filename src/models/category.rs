use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub category_id: i32,
    pub category_name: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_categories(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Category>, Error> {
    let base_query =
        "from categories c where c.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "category_name"
    } else {
        "c.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "c.category_id, c.category_name, c.created_at",
        base_query: &base_query,
        search_columns: vec!["c.category_id::varchar", "c.category_name"],
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

    let categories: Vec<Category> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| Category {
            category_id: row.get("category_id"),
            category_name: row.get("category_name"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: categories,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct CategoryRequest {
    pub category_name: String,
}

pub async fn add_category(
    data: &CategoryRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into categories (category_name) values ($1)",
            &[&data.category_name],
        )
        .await?;
    Ok(())
}

pub async fn get_category_by_id(category_id: i32, client: &Client) -> Option<Category> {
    let result = client
        .query_one(
            "select c.category_id, c.category_name, c.created_at from categories c where c.deleted_at is null and c.category_id = $1",
            &[&category_id],
        )
        .await;

    match result {
        Ok(row) => Some(Category {
            category_id: row.get("category_id"),
            category_name: row.get("category_name"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_category(
    category_id: i32,
    data: &CategoryRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update categories set category_name = $1 where category_id = $2",
            &[&data.category_name, &category_id],
        )
        .await?;

    Ok(())
}

pub async fn delete_category(
    category_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update categories set deleted_at = CURRENT_TIMESTAMP where category_id = $1",
            &[&category_id],
        )
        .await?;

    Ok(())
}
