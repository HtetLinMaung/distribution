use std::sync::Arc;

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio_postgres::Client;

use crate::{
    models::order::{self, OrderRequest},
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[post("/api/orders")]
pub async fn add_order(
    req: HttpRequest,
    body: web::Json<OrderRequest>,
    data: web::Data<Arc<Mutex<Client>>>,
) -> impl Responder {
    let mut client = data.lock().await;
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 3 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let user_id: i32 = parsed_values[0].parse().unwrap();
    // let role: &str = parsed_values[1];

    match order::add_order(&body, user_id, &mut client).await {
        Ok(order_id) => {
            if order_id == 0 {
                return HttpResponse::BadRequest().json(DataResponse {
                    code: 400,
                    message: String::from(
                        "We're sorry, the products you requested are not available!",
                    ),
                    data: Some(order_id),
                });
            }
            HttpResponse::Ok().json(DataResponse {
                code: 200,
                message: String::from("Successful."),
                data: Some(order_id),
            })
        }
        Err(err) => {
            println!("{:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding order to database!"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct GetOrdersQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub from_amount: Option<f64>,
    pub to_amount: Option<f64>,
    pub status: Option<String>,
}

#[get("/api/orders")]
pub async fn get_orders(
    req: HttpRequest,
    data: web::Data<Arc<Mutex<Client>>>,
    query: web::Query<GetOrdersQuery>,
) -> impl Responder {
    let client = data.lock().await;
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 3 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let user_id: &str = parsed_values[0];
    let user_id: i32 = user_id.parse().unwrap();
    let role: &str = parsed_values[1];

    match order::get_orders(
        &query.search,
        query.page,
        query.per_page,
        &query.from_date,
        &query.to_date,
        &query.from_amount,
        &query.to_amount,
        &query.status,
        user_id,
        role,
        &client,
    )
    .await
    {
        Ok(item_result) => HttpResponse::Ok().json(PaginationResponse {
            code: 200,
            message: String::from("Successful."),
            data: item_result.data,
            total: item_result.total,
            page: item_result.page,
            per_page: item_result.per_page,
            page_counts: item_result.page_counts,
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving orders: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all orders from database"),
            })
        }
    }
}
