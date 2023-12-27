use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};
use std::{fs, option::Option, path::Path};

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
    pub ward_id: i32,
    pub ward_name: String,
    pub weekdays: Vec<Weekdays>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Weekdays {
    pub weekday_id: i32,
    pub weekday_name: String,
}

pub async fn get_shops(
    user_id: i32,
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    weekdays: &Option<String>,
    client: &Client,
) -> Result<PaginationResult<Shop>, Error> {
    let mut base_query = if role == "Distributor" {
        format!(
            "from 
        users u, user_wards uw, shops s, shop_weekdays sw, wards w
        where u.user_id = uw.user_id and uw.ward_id=s.ward_id and s.shop_id=sw.shop_id
        and w.ward_id=s.ward_id
        and s.deleted_at is null and u.user_id={}",
            user_id
        )
    } else {
        "from shops s where s.deleted_at is null".to_string()
    };
    if role == "Distributor"
        && weekdays.is_some()
        && !weekdays.as_ref().unwrap_or(&String::new()).is_empty()
    {
        base_query += &format!(" AND sw.weekday_id IN ({})", weekdays.as_ref().unwrap());
    }
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "s.shop_name"
    } else {
        "s.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "s.shop_id, s.shop_name, s.address, COALESCE(s.latitude,0.0)::text as latitude, COALESCE(s.longitude,0.0)::text as longitude, image_url, w.ward_id, w.ward_name, s.created_at",
        base_query: &base_query,
        search_columns: vec!["s.shop_id::varchar", "s.shop_name", "s.address", "w.ward_name"],
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

    let rows = client.query(&result.query, &params_slice).await?;
    let mut shops: Vec<Shop> = vec![];
    for row in &rows {
        let shop_id: i32 = row.get("shop_id");

        let weekdays_rows = client.query("select sw.weekday_id, w.weekday_name from shop_weekdays sw join weekdays w on w.weekday_id = sw.weekday_id where sw.shop_id = $1", &[&shop_id]).await?;
        let latitude_str: &str = row.get("latitude");
        let longitude_str: &str = row.get("longitude");
        shops.push(Shop {
            shop_id: shop_id,
            shop_name: row.get("shop_name"),
            address: row.get("address"),
            latitude: latitude_str.parse().unwrap(),
            longitude: longitude_str.parse().unwrap(),
            image_url: row.get("image_url"),
            ward_id: row.get("ward_id"),
            ward_name: row.get("ward_name"),
            weekdays: weekdays_rows
                .iter()
                .map(|row: &tokio_postgres::Row| Weekdays {
                    weekday_id: row.get("weekday_id"),
                    weekday_name: row.get("weekday_name"),
                })
                .collect(),
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
            "select s.shop_id, s.shop_name, s.address, COALESCE(s.latitude,0.0)::text as latitude, COALESCE(s.longitude,0.0)::text as longitude, image_url, w.ward_id, w.ward_name, s.created_at 
            from shops s, wards w where w.ward_id=s.ward_id and s.deleted_at is null and s.shop_id = $1",
            &[&shop_id],
        )
        .await;

        let weekdays_rows = match client.query("select sw.weekday_id, w.weekday_name 
        from shop_weekdays sw join weekdays w on w.weekday_id = sw.weekday_id 
        where sw.shop_id = $1", &[&shop_id]).await
    {
        Ok(rows) => rows,
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    };

    match result {
        Ok(row) => {
            let latitude_str: &str = row.get("latitude");
            let longitude_str: &str = row.get("longitude");
            Some(Shop {
                shop_id: shop_id,
                shop_name: row.get("shop_name"),
                address: row.get("address"),
                latitude: latitude_str.parse().unwrap(),
                longitude: longitude_str.parse().unwrap(),
                image_url: row.get("image_url"),
                ward_id: row.get("ward_id"),
                ward_name: row.get("ward_name"),
                weekdays: weekdays_rows
                    .iter()
                    .map(|row: &tokio_postgres::Row| Weekdays {
                        weekday_id: row.get("weekday_id"),
                        weekday_name: row.get("weekday_name"),
                    })
                    .collect(),
                created_at: row.get("created_at"),
            })
        }
        Err(_) => None,
    }
}

#[derive(Debug, Deserialize)]
pub struct ShopRequest {
    pub shop_name: String,
    pub address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub image_url: String,
    pub ward_id: i32,
    pub weekdays: Vec<i32>,
}

pub async fn add_shop(
    data: &ShopRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!(
        "INSERT INTO shops (shop_name, address, latitude, longitude, image_url, ward_id) VALUES ($1, $2, {}, {}, $3, $4) RETURNING shop_id",
        data.latitude.map_or("NULL".to_string(), |v| v.to_string()),
        data.longitude.map_or("NULL".to_string(), |v| v.to_string())
    );
    
    let row = client
        .query_one(
            &query,
            &[&data.shop_name, &data.address, &data.image_url, &data.ward_id],
        )
        .await?;
    let id: i32 = row.get("shop_id");
    for weekday_id in &data.weekdays {
        client
            .execute(
                "INSERT INTO shop_weekdays (shop_id, weekday_id) VALUES ($1, $2)",
                &[&id, &weekday_id],
            )
            .await?;
    }
    Ok(())
}

pub async fn update_shop(
    shop_id: i32,
    old_image_url: &str,
    data: &ShopRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!(
        "UPDATE shops SET shop_name = $1, address = $2, latitude={}, longitude={}, image_url=$3, ward_id=$4 WHERE shop_id = $5",
        data.latitude.map_or("NULL".to_string(), |v| v.to_string()),
        data.longitude.map_or("NULL".to_string(), |v| v.to_string())
    );
    client
        .execute(
            &query,
            &[&data.shop_name, &data.address, &data.image_url, &data.ward_id, &shop_id],
        )
        .await?;
    client
        .execute(
            "DELETE FROM shop_weekdays WHERE shop_id = $1",
            &[&shop_id],
        )
        .await?;

    for weekday_id in &data.weekdays {
        client
            .execute(
                "INSERT INTO shop_weekdays (shop_id, weekday_id) VALUES ($1, $2)",
                &[&shop_id, &weekday_id],
            )
            .await?;
    }

    if old_image_url != &data.image_url {
        match fs::remove_file(old_image_url) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };

        let path = Path::new(&old_image_url);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        match fs::remove_file(format!("{stem}_original.{extension}")) {
            Ok(_) => println!("Original file deleted successfully!"),
            Err(e) => println!("Error deleting original file: {}", e),
        };
    }
    Ok(())
}

pub async fn delete_shop(shop_id: i32, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update shops set deleted_at = CURRENT_TIMESTAMP where shop_id = $1",
            &[&shop_id],
        )
        .await?;

    Ok(())
}
