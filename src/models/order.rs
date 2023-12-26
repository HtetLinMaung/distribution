use serde::Deserialize;
use tokio_postgres::{Client, Error};

#[derive(Deserialize)]
pub struct OrderRequest {
    pub shop_id: i32,
    pub order_details: Vec<OrderDetail>,
}

#[derive(Deserialize)]
pub struct OrderDetail {
    pub price_id: i32,
    pub quantity: i32,
}

pub async fn add_order(
    data: &OrderRequest,
    user_id: i32,
    client: &mut Client,
) -> Result<i32, Error> {
    let transaction = client.transaction().await?;

    // Check available quantities first
    for order_detail in &data.order_details {
        let row = transaction
            .query_one(
                "SELECT remaining_quantity FROM product_prices WHERE price_id = $1 AND deleted_at IS NULL FOR UPDATE",
                &[&order_detail.price_id],
            )
            .await?;

        let remaining_quantity: i32 = row.get("remaining_quantity");
        if order_detail.quantity > remaining_quantity {
            transaction.rollback().await?;
            return Ok(0);
        }
    }

    // Insert the order
    let row = transaction
        .query_one(
            "INSERT INTO orders (shop_id, user_id, status, total_amount) VALUES ($1, $2, 'Pending', 0.0) RETURNING order_id",
            &[&data.shop_id, &user_id],
        )
        .await?;
    let order_id: i32 = row.get("order_id");

    // Process each order detail
    for order_detail in &data.order_details {
        // Update remaining quantities
        transaction
            .execute(
                "UPDATE product_prices SET remaining_quantity = remaining_quantity - $1 WHERE price_id = $2 AND deleted_at IS NULL",
                &[&order_detail.quantity, &order_detail.price_id],
            )
            .await?;

        // Get discount ID
        let discount_id = match transaction
            .query_one(
                "SELECT discount_id::text FROM product_discounts WHERE price_id = $1 AND deleted_at IS NULL",
                &[&order_detail.price_id],
            )
            .await
        {
            Ok(row) => row.get("discount_id"),
            Err(_) => "null".to_string(),
        };

        // Insert order details
        transaction
            .execute(
                &format!("INSERT INTO order_details (order_id, price_id, quantity, price_at_order, discount_id) VALUES ($1, $2, $3, (SELECT price FROM product_prices WHERE price_id = $4), {discount_id})"),
                &[
                    &order_id,
                    &order_detail.price_id,
                    &order_detail.quantity,
                    &order_detail.price_id,
                ],
            )
            .await?;
    }

    transaction
        .execute(
            "update orders set total_amount = (select coalesce(sum(price_at_order * quantity), 0.0) from order_details where order_id = $1) where order_id = $2",
            &[&order_id, &order_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(order_id)
}
