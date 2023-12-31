use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};
use std::{fs, option::Option, path::Path};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub product_id: i32,
    pub product_name: String,
    pub image_url: String,
    pub brand_id: i32,
    pub brand_name: String,
    pub categories: Vec<Categories>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Categories {
    pub category_id: i32,
    pub category_name: String,
}

pub async fn get_products(
    category_id: Option<usize>,
    brand_id: Option<usize>,
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Product>, Error> {
    let mut base_query = "from products p, brands b, categories c, product_categories pc  
    where p.product_id=pc.product_id and pc.category_id=c.category_id and p.brand_id=b.brand_id and p.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    if let Some(category_id) = category_id {
        base_query += &format!(" AND c.category_id={}", category_id);
    }
    if let Some(brand_id) = brand_id {
        base_query += &format!(" AND b.brand_id={}", brand_id);
    }
    let order_options = if role == "Distributor" {
        "p.product_name"
    } else {
        "p.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "distinct p.product_id, p.product_name, p.image_url, b.brand_id, b.brand_name, p.created_at",
        base_query: &base_query,
        search_columns: vec!["p.product_id::varchar", "p.product_name", "b.brand_name", "c.category_name"],
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
    let mut products: Vec<Product> = vec![];
    for row in &rows {
        let product_id: i32 = row.get("product_id");

        let categories_rows = client.query("select c.category_id, c.category_name from product_categories pc join categories c on pc.category_id = c.category_id where pc.product_id = $1", &[&product_id]).await?;
        products.push(Product {
            product_id: product_id,
            product_name: row.get("product_name"),
            image_url: row.get("image_url"),
            brand_id: row.get("brand_id"),
            brand_name: row.get("brand_name"),
            categories: categories_rows
                .iter()
                .map(|row: &tokio_postgres::Row| Categories {
                    category_id: row.get("category_id"),
                    category_name: row.get("category_name"),
                })
                .collect(),
            created_at: row.get("created_at"),
        });
    }

    Ok(PaginationResult {
        data: products,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_product_by_id(product_id: i32, client: &Client) -> Option<Product> {
    let result = client
        .query_one(
            "select s.product_id, s.product_name, image_url, b.brand_id, b.brand_name, s.created_at 
            from products s, brands b where b.brand_id=s.brand_id and s.deleted_at is null and s.product_id = $1",
            &[&product_id],
        )
        .await;

        let categories_rows = match client.query("select pc.category_id, c.category_name 
        from product_categories pc join categories c on c.category_id = pc.category_id 
        where pc.product_id = $1", &[&product_id]).await
    {
        Ok(rows) => rows,
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    };

    match result {
        Ok(row) => {
            Some(Product {
                product_id: product_id,
                product_name: row.get("product_name"),
                image_url: row.get("image_url"),
                brand_id: row.get("brand_id"),
                brand_name: row.get("brand_name"),
                categories: categories_rows
                    .iter()
                    .map(|row: &tokio_postgres::Row| Categories {
                        category_id: row.get("category_id"),
                        category_name: row.get("category_name"),
                    })
                    .collect(),
                created_at: row.get("created_at"),
            })
        }
        Err(_) => None,
    }
}

#[derive(Debug, Deserialize)]
pub struct ProductRequest {
    pub product_name: String,
    pub image_url: String,
    pub brand_id: i32,
    pub categories: Vec<i32>,
}

pub async fn add_product(
    data: &ProductRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let row = client
        .query_one(
            "INSERT INTO products (product_name, image_url, brand_id) VALUES ($1, $2, $3) RETURNING product_id",
            &[&data.product_name, &data.image_url, &data.brand_id],
        )
        .await?;
    let id: i32 = row.get("product_id");
    for category_id in &data.categories {
        client
            .execute(
                "INSERT INTO product_categories (product_id, category_id) VALUES ($1, $2)",
                &[&id, &category_id],
            )
            .await?;
    }
    Ok(())
}

pub async fn update_product(
    product_id: i32,
    old_image_url: &str,
    data: &ProductRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "UPDATE products SET product_name = $1, image_url=$2, brand_id=$3 WHERE product_id = $4",
            &[&data.product_name, &data.image_url, &data.brand_id, &product_id],
        )
        .await?;
    client
        .execute(
            "DELETE FROM product_categories WHERE product_id = $1",
            &[&product_id],
        )
        .await?;

    for category_id in &data.categories {
        client
            .execute(
                "INSERT INTO product_categories (product_id, category_id) VALUES ($1, $2)",
                &[&product_id, &category_id],
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

pub async fn delete_product(product_id: i32, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update products set deleted_at = CURRENT_TIMESTAMP where product_id = $1",
            &[&product_id],
        )
        .await?;

    Ok(())
}
