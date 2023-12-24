use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ward {
    pub ward_id: i32,
    pub township_id: i32,
    pub ward_name: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_wards(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Ward>, Error> {
    let base_query =
        "from wards b where b.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "ward_name"
    } else {
        "b.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "b.ward_id, b.township_id, b.ward_name,b.created_at",
        base_query: &base_query,
        search_columns: vec!["b.ward_id::varchar", "b.ward_name"],
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

    let wards: Vec<Ward> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| Ward {
            ward_id: row.get("ward_id"),
            township_id: row.get("township_id"),
            ward_name: row.get("ward_name"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: wards,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct WardRequest {
    pub ward_name: String,
    pub township_id: i32,
}

pub async fn add_ward(
    data: &WardRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into wards (ward_name,township_id) values ($1,$2)",
            &[&data.ward_name, &data.township_id],
        )
        .await?;
    Ok(())
}

pub async fn get_ward_by_id(ward_id: i32, client: &Client) -> Option<Ward> {
    let result = client
        .query_one(
            "select ward_id,township_id, ward_name,created_at from wards  where deleted_at is null and ward_id = $1",
            &[&ward_id],
        )
        .await;

    match result {
        Ok(row) => Some(Ward {
            ward_id: row.get("ward_id"),
            township_id: row.get("township_id"),
            ward_name: row.get("ward_name"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_ward(
    ward_id: i32,
    data: &WardRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update wards set ward_name = $1, township_id =$2 where ward_id = $3",
            &[&data.ward_name, &data.township_id, &ward_id],
        )
        .await?;

    Ok(())
}

pub async fn delete_ward(
    ward_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update wards set deleted_at = CURRENT_TIMESTAMP where ward_id = $1",
            &[&ward_id],
        )
        .await?;

    Ok(())
}
