use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Township {
    pub township_id: i32,
    pub township_name: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_townships(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Township>, Error> {
    let base_query =
        "from townships b where b.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "township_name"
    } else {
        "b.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "b.township_id, b.township_name,b.created_at",
        base_query: &base_query,
        search_columns: vec!["b.id::varchar", "b.township_name"],
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

    let townships: Vec<Township> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| Township {
            township_id: row.get("township_id"),
            township_name: row.get("township_name"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: townships,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct TownshipRequest {
    pub township_name: String,
}

pub async fn add_township(
    data: &TownshipRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into townships (township_name) values ($1)",
            &[&data.township_name],
        )
        .await?;
    Ok(())
}

pub async fn get_township_by_id(township_id: i32, client: &Client) -> Option<Township> {
    let result = client
        .query_one(
            "select township_id, township_name,created_at from townships  where deleted_at is null and township_id = $1",
            &[&township_id],
        )
        .await;

    match result {
        Ok(row) => Some(Township {
            township_id: row.get("township_id"),
            township_name: row.get("township_name"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_township(
    township_id: i32,
    data: &TownshipRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update townships set township_name = $1 where township_id = $2",
            &[&data.township_name, &township_id],
        )
        .await?;

    Ok(())
}

pub async fn delete_township(
    township_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update townships set deleted_at = CURRENT_TIMESTAMP where township_id = $1",
            &[&township_id],
        )
        .await?;

    Ok(())
}
