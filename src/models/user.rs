use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)] // Add Debug derive
pub struct UserWard {
    pub ward_id: i32,
    pub ward_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub userid: i32,
    pub fullname: String,
    pub username: String,
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub wards: Vec<UserWard>,
}


pub async fn get_user(username: &str, client: &Client) -> Option<User> {
    let result = client
        .query_one(
            "select user_id,full_name,username,password,role,created_at from users  where username = $1 and deleted_at is null",
            &[&username],
        )
        .await;

    match result {
        Ok(row) => Some(User {
            userid: row.get("user_id"),
            fullname: row.get("full_name"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            wards: Vec::new(), 
        }),
        Err(_) => None,
    }
}

#[derive(Deserialize)]
pub struct AddUserRequest {
    pub fullname: String,
    pub username: String,
    pub password: String,
    pub role: String,
    pub ward_ids: Vec<i32>,

}



pub async fn add_user(
    data: &AddUserRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_password = hash(&data.password, DEFAULT_COST)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    // Insert user into the users table
    let user_insert_query = "
        insert into users (full_name, username, password, role)
        values ($1, $2, $3, $4)
        RETURNING user_id
    ";
    let user_id: i32 = client
        .query_one(
            user_insert_query,
            &[&data.fullname, &data.username, &hashed_password, &data.role],
        )
        .await?
        .get("user_id");

    // Insert user-ward relationship into the user_wards table
    let user_wards_insert_query = "
        insert into user_wards (user_id, ward_id)
        values ($1, $2)
    ";
    for ward_id in &data.ward_ids {
        client
            .execute(user_wards_insert_query, &[&user_id, &ward_id])
            .await?;
    }

    Ok(())
}



pub async fn get_users(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<User>, Error> {
    let base_query =
        "from users u where u.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "Distributor" {
        "full_name"
    } else {
        "u.created_at desc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "u.user_id, u.full_name,u.username,u.password, u.role, u.created_at",
        base_query: &base_query,
        search_columns: vec!["u.user_id, u.full_name,u.username, u.role"],
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
    let mut users: Vec<User> = vec![];
    for row in &rows {
        let user_id = row.get("user_id");
        let wards_rows =  client
        .query(
            "SELECT uw.ward_id, w.ward_name FROM user_wards uw JOIN wards w ON w.ward_id = uw.ward_id WHERE uw.user_id = $1",
            &[&user_id],
        )
        .await?;
        users.push(
            User {
            userid: user_id,
            fullname: row.get("full_name"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            wards: wards_rows
                .iter()
                .map(|row| UserWard {
                    ward_id: row.get("ward_id"),
                    ward_name: row.get("ward_name"),
                })
                .collect(),
            }
        );
    }
    Ok(PaginationResult {
        data: users,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_user_by_id(user_id: i32, client: &Client) -> Option<User> {
    let result = client.query_one("select user_id,full_name, username, password, role, created_at from users  where deleted_at is null  and user_id = $1", &[&user_id]).await;
    let wards_rows = match client
        .query(
            "SELECT uw.ward_id, w.ward_name FROM user_wards uw JOIN wards w ON w.ward_id = uw.ward_id WHERE uw.user_id = $1",
            &[&user_id],
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
        Ok(row) => Some(User {
            userid: row.get("user_id"),
            fullname: row.get("full_name"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            wards: wards_rows
                    .iter()
                    .map(|row| UserWard {
                        ward_id: row.get("ward_id"),
                        ward_name: row.get("ward_name"),
                    })
                    .collect(),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub fullname: String,
    pub password: String,
    pub role: String,
    pub ward_ids: Vec<i32>,
}

pub async fn update_user(
    old_password: &str,
    data: &UpdateUserRequest,
    user_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let password: &str = &data.password;
    let mut hashed_password = password.to_string();

    if password != old_password {
        hashed_password = hash(&data.password.as_str(), DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;
    }

    client
        .execute(
            "update users set full_name = $1, password = $2, role = $3 where user_id = $4",
            &[
                &data.fullname,
                &hashed_password,
                &data.role,
                &user_id,
            ],
        )
        .await?;
    client.execute("delete from user_wards where user_id = $1",&[&user_id],).await?;

    let user_wards_insert_query = "insert into user_wards (user_id, ward_id) values ($1, $2) ";
    for ward_id in &data.ward_ids {
        client
            .execute(user_wards_insert_query, &[&user_id, &ward_id])
            .await?;
    }
    Ok(())
}

pub async fn delete_user(user_id: i32, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update users set deleted_at = CURRENT_TIMESTAMP where user_id = $1 and deleted_at is null",
            &[&user_id],
        )
        .await?;
    client
        .execute(
            "delete from user_wards where user_id = $1",
            &[&user_id],
        )
        .await?;
    Ok(())
}

pub async fn user_exists(username: &str, client: &Client) -> Result<bool, Error> {
    // Execute a query to check if the username exists in the users table
    let row = client
        .query_one(
            "SELECT username FROM users WHERE username = $1 and deleted_at is null",
            &[&username],
        )
        .await;

    // Return whether the user exists
    Ok(row.is_ok())
}
