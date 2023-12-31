use chrono::{NaiveDateTime,NaiveDate};
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)] // Add Debug derive
pub struct ProductPrice {
    pub price_id: i32,
    pub product_id: i32,
    pub product_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Discount {
    pub discount_id: i32,
    pub discount_name: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub start_date: String,
    pub end_date: String,
    pub min_quantity: i32,
    pub max_quantity: i32,
    pub conditions: String,
    pub created_at: NaiveDateTime,
    pub product_prices: Vec<ProductPrice>,

}

pub async fn get_discounts(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Discount>, Error> {
    let base_query =
        "from discounts d where d.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "discount_name"
    } else {
        "d.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "d.discount_id,d.discount_name, d.discount_type,d.discount_value::text as discount_value,d.start_date::text as start_date,d.end_date::text as end_date,d.min_quantity,d.max_quantity,d.conditions,d.created_at",
        base_query: &base_query,
        search_columns: vec!["d.discount_id::varchar", "d.discount_name","d.discount_type"],
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
        let mut discounts: Vec<Discount> = vec![];
        for row in &rows {
            let discount_value_str: &str = row.get("discount_value");
            let start_date_str: &str = row.get("start_date");
            let end_date_str: &str = row.get("end_date");
            discounts.push(
                Discount {
                    discount_id: row.get("discount_id"),
                    discount_name: row.get("discount_name"),
                    discount_type: row.get("discount_type"),
                    discount_value: discount_value_str.parse().unwrap(),
                    start_date: start_date_str.parse().unwrap(),
                    end_date: end_date_str.parse().unwrap(),
                    min_quantity: row.get("min_quantity"),
                    max_quantity: row.get("max_quantity"),
                    conditions: row.get("conditions"),
                    created_at: row.get("created_at"),
                    product_prices: Vec::new(),

                }
            );
        }

    Ok(PaginationResult {
        data: discounts,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct DiscountRequest {
    pub discount_name: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_quantity: i32,
    pub max_quantity: i32,
    pub conditions: String,
    pub price_ids: Vec<i32>,

}

pub async fn add_discount(
    data: &DiscountRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    
   let discount_insert_query = format!("insert into discounts (discount_name,discount_type,discount_value,start_date,end_date,min_quantity,max_quantity,conditions) values ($1,$2,{},$3,$4,$5,$6,$7)  RETURNING discount_id",data.discount_value);
   let discount_id: i32 = client
   .query_one(
    &discount_insert_query,
       &[&data.discount_name, &data.discount_type,  &data.start_date, &data.end_date, &data.min_quantity, &data.max_quantity, &data.conditions ],
   )
   .await?
   .get("discount_id"); 
    let product_discounts_insert_query = "insert into product_discounts (price_id, discount_id) values ($1, $2)";
    for price_id in &data.price_ids {
        client.execute(product_discounts_insert_query, &[&price_id,&discount_id]).await?;
    }
    Ok(())
}

pub async fn get_discount_by_id(discount_id: i32, client: &Client) -> Option<Discount> {
    let result = client.query_one("select discount_id,discount_name, discount_type, discount_value::text as discount_value, start_date::text as start_date, end_date::text as end_date, min_quantity, max_quantity, conditions, created_at from discounts  where deleted_at is null  and discount_id = $1", &[&discount_id]).await;
    let product_price_rows = match client
        .query(
            "select pd.price_id,p.product_id, p.product_name
            from product_discounts pd 
            join discounts d ON d.discount_id = pd.discount_id
            join product_prices pp ON pp.price_id = pd.price_id
            join products p ON p.product_id = pp.product_id where pd.discount_id = $1",
            &[&discount_id],
        )
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    };
    match result {
        Ok(row) => {
            let discount_value_str: &str = row.get("discount_value");
            let start_date_str: &str = row.get("start_date");
            let end_date_str: &str = row.get("end_date");

            Some(Discount {
                discount_id: row.get("discount_id"),
                discount_name: row.get("discount_name"),
                discount_type: row.get("discount_type"),
                discount_value: discount_value_str.parse().unwrap(),
                start_date: start_date_str.parse().unwrap(),
                end_date: end_date_str.parse().unwrap(),
                min_quantity: row.get("min_quantity"),
                max_quantity: row.get("max_quantity"),
                conditions: row.get("conditions"),
                created_at: row.get("created_at"),
                product_prices: product_price_rows
                        .iter()
                        .map(|row| ProductPrice {
                            price_id: row.get("price_id"),
                            product_id: row.get("product_id"),
                            product_name: row.get("product_name"),
                        })
                        .collect(),
            })
        },
        Err(err) => {
            println!("{:?}", err);
            None
        }
        
    }
}

pub async fn update_discount(
    data: &DiscountRequest,
    discount_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
 let query = format!("update discounts set discount_name = $1, discount_type = $2, discount_value = {},
 start_date = $3, end_date = $4, min_quantity = $5, max_quantity = $6, conditions = $7 where discount_id = $8",data.discount_value);
    client
        .execute(
            &query,
            &[
                &data.discount_name,
                &data.discount_type,
                &data.start_date,
                &data.end_date,
                &data.min_quantity,
                &data.max_quantity,
                &data.conditions,
                &discount_id,
            ],
        )
        .await?;
    client.execute("delete from product_discounts where discount_id = $1",&[&discount_id],).await?;

    let product_discounts_insert_query = "insert into product_discounts (price_id, discount_id) values ($1, $2) ";
    for price_id in &data.price_ids {
        client
            .execute(product_discounts_insert_query, &[&price_id, &discount_id])
            .await?;
    }
    Ok(())
}

pub async fn delete_discount(
    discount_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update discounts set deleted_at = CURRENT_TIMESTAMP where discount_id = $1",
            &[&discount_id],
        )
        .await?;
    client
    .execute(
        "delete from product_discounts where discount_id = $1",
        &[&discount_id],
    )
    .await?;
    Ok(())
}
