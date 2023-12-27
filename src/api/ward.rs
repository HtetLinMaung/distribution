use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;
use tokio::sync::Mutex;

use crate::{
    models::ward::{self, WardRequest},
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetWardsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/wards")]
pub async fn get_wards(
    req: HttpRequest,
    data: web::Data<Arc<Mutex<Client>>>,
    query: web::Query<GetWardsQuery>,
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

    // let user_id: i32 = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    match ward::get_wards(
        &query.search,
        query.page,
        query.per_page,
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
            println!("Error retrieving wards: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all wards from database"),
            })
        }
    }
}

#[post("/api/wards")]
pub async fn add_ward(
    req: HttpRequest,
    body: web::Json<WardRequest>,
    data: web::Data<Arc<Mutex<Client>>>,
) -> HttpResponse {
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

    // let user_id: i32 = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    if role != "Admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.ward_name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Ward Name must not be empty!"),
        });
    }

    match ward::add_ward(&body, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Ward added successfully"),
        }),
        Err(e) => {
            eprintln!("Ward adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding ward!"),
            });
        }
    }
}

#[get("/api/wards/{ward_id}")]
pub async fn get_ward_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    data: web::Data<Arc<Mutex<Client>>>,
) -> HttpResponse {
    let client = data.lock().await;
    let ward_id = path.into_inner();
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

    let role: &str = parsed_values[1];

    if role != "Admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match ward::get_ward_by_id(ward_id, &client).await {
        Some(c) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Ward fetched successfully."),
            data: Some(c),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Ward not found!"),
        }),
    }
}

#[put("/api/wards/{ward_id}")]
pub async fn update_ward(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<WardRequest>,
    data: web::Data<Arc<Mutex<Client>>>,
) -> HttpResponse {
    let client = data.lock().await;
    let ward_id = path.into_inner();
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

    let role: &str = parsed_values[1];

    if role != "Admin"  {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.ward_name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Ward Name must not be empty!"),
        });
    }

    match ward::get_ward_by_id(ward_id, &client).await {
        Some(_) => match ward::update_ward(ward_id, &body, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 200,
                message: String::from("Ward updated successfully"),
            }),
            Err(e) => {
                eprintln!("Ward updating error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error updating ward!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Ward not found!"),
        }),
    }
}

#[delete("/api/wards/{ward_id}")]
pub async fn delete_ward(
    req: HttpRequest,
    path: web::Path<i32>,
    data: web::Data<Arc<Mutex<Client>>>,
) -> HttpResponse {
    let client = data.lock().await;
    let ward_id = path.into_inner();
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

    let role: &str = parsed_values[1];

    if role != "Admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match ward::get_ward_by_id(ward_id, &client).await {
        Some(_) => match ward::delete_ward(ward_id, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("Ward deleted successfully"),
            }),
            Err(e) => {
                eprintln!("Ward deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting ward!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Ward not found!"),
        }),
    }
}
