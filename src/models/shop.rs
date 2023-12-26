use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Shop {
    pub shop_id: i32,
    pub shop_name: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub image_url: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_shops(
    user_id: i32,
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    weekdays:&Option<String>,
    client: &Client,
) -> Result<PaginationResult<Shop>, Error> {
    let mut base_query = if role == "Distributor" {
        format!("from 
        users u, user_wards uw, shops s, shop_weekdays sw
        where u.user_id = uw.user_id and uw.ward_id=s.ward_id and s.shop_id=sw.shop_id
        and s.deleted_at is null and u.user_id={}", user_id)
    } else {
        "from shops s where s.deleted_at is null".to_string()
    };
    if role == "Distributor" && weekdays.is_some() && !weekdays.as_ref().unwrap_or(&String::new()).is_empty() {
        base_query += &format!(" AND sw.weekday_id IN ({})", weekdays.as_ref().unwrap());
    }
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "s.shop_name"
    } else {
        "s.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "s.shop_id, s.shop_name, s.address, COALESCE(s.latitude,0.0)::text as latitude, COALESCE(s.longitude,0.0)::text as longitude, image_url, s.created_at",
        base_query: &base_query,
        search_columns: vec!["s.shop_id::varchar", "s.shop_name", "s.address"],
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
    let mut shops : Vec<Shop> = vec![];
    for row in &rows {
        let latitude_str: &str = row.get("latitude");
        let longitude_str: &str = row.get("longitude");
        shops.push(Shop{
            shop_id: row.get("shop_id"),
            shop_name: row.get("shop_name"),
            address: row.get("address"),
            latitude: latitude_str.parse().unwrap(),
            longitude: longitude_str.parse().unwrap(),
            image_url: row.get("image_url"),
            created_at: row.get("created_at"),
        });
    }

    Ok(PaginationResult {
        data: shops,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_shop_by_id(shop_id: i32, client: &Client) -> Option<Shop> {
    let result = client
        .query_one(
            "select s.shop_id, s.shop_name, s.address, COALESCE(s.latitude,0.0)::text as latitude, COALESCE(s.longitude,0.0)::text as longitude, image_url, s.created_at from shops s where s.deleted_at is null and s.shop_id = $1",
            &[&shop_id],
        )
        .await;

    match result {
        Ok(row) => {
            let latitude_str: &str = row.get("latitude");
            let longitude_str: &str = row.get("longitude");
            Some(Shop {
            shop_id: row.get("shop_id"),
            shop_name: row.get("shop_name"),
            address: row.get("address"),
            latitude: latitude_str.parse().unwrap(),
            longitude: longitude_str.parse().unwrap(),
            image_url: row.get("image_url"),
            created_at: row.get("created_at"),
        })},
        Err(_) => None,
    }
}

// #[derive(Debug, Deserialize)]
// pub struct ShopRequest {
//     pub name: String,
//     pub description: String
// }

// pub async fn add_shop(
//     data: &ShopRequest,
//     client: &Client,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     client
//         .execute(
//             "insert into shops (name, description) values ($1, $2)",
//             &[&data.name, &data.description],
//         )
//         .await?;
//     Ok(())
// }



// pub async fn update_shop(
//     shop_id: i32,
//     data: &ShopRequest,
//     client: &Client,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     client
//         .execute(
//             "update shops set name = $1, description = $2 where id = $3",
//             &[&data.name, &data.description, &shop_id],
//         )
//         .await?;

//     Ok(())
// }

// pub async fn delete_shop(
//     shop_id: i32,
//     client: &Client,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     client
//         .execute(
//             "update shops set deleted_at = CURRENT_TIMESTAMP where id = $1",
//             &[&shop_id],
//         )
//         .await?;

//     Ok(())
// }
