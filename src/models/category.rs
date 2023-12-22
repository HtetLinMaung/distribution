use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: String,
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

    let order_options = if role == "Sale" {
        "name"
    } else {
        "c.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "c.id, c.name, c.description, c.created_at",
        base_query: &base_query,
        search_columns: vec!["c.id::varchar", "c.name", "c.description"],
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
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
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
    pub name: String,
    pub description: String
}

pub async fn add_category(
    data: &CategoryRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into categories (name, description) values ($1, $2)",
            &[&data.name, &data.description],
        )
        .await?;
    Ok(())
}

pub async fn get_category_by_id(category_id: i32, client: &Client) -> Option<Category> {
    let result = client
        .query_one(
            "select c.id, c.name, c.description, c.created_at from categories c where c.deleted_at is null and c.id = $1",
            &[&category_id],
        )
        .await;

    match result {
        Ok(row) => Some(Category {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
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
            "update categories set name = $1, description = $2 where id = $3",
            &[&data.name, &data.description, &category_id],
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
            "update categories set deleted_at = CURRENT_TIMESTAMP where id = $1",
            &[&category_id],
        )
        .await?;

    Ok(())
}
