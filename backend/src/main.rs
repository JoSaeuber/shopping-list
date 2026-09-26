use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};use sqlx::SqlitePool;
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::env;

use serde::{Deserialize, Serialize};

// This struct represents a full item in the database.
// Serialize converts it TO JSON when we send it to the client.
#[derive(Serialize)]
struct Item {
    id: i64,
    name: String,
    is_checked: bool,
}

// This struct represents the payload the client sends to create an item.
// Deserialize converts it FROM JSON.
#[derive(Deserialize)]
struct CreateItem {
    name: String,
}

#[tokio::main]
async fn main(){
    // Load variables from .env
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL mus be set in .env");

    // Initialize the database connection pool
    println!("Connecting to database...");
    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to SQLite");

    // Add the pool to Axum Router state
    // Replace your old Router::new() block with this:
    let app = Router::new()
    .route("/", get(hello_world))
    .route("/db", get(check_db))
    .route("/items", post(create_item).get(get_items)) // New routes
    .with_state(pool);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server is running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Hello World! The backend is alive."
}

async fn check_db(State(pool): State<SqlitePool>) -> String {    // Run a simple query to verify the connection
    let result: (String,) = sqlx::query_as("SELECT sqlite_version()")
        .fetch_one(&pool)
        .await
        .unwrap();
        
    format!("Database connected! SQLite version: {}", result.0)
}

// POST handler to create a new item
async fn create_item(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), StatusCode> {
    
    // The RETURNING clause lets us get the generated ID back immediately
    let item = sqlx::query_as!(
        Item,
        "INSERT INTO items (name) VALUES ($1) RETURNING id, name, is_checked",
        payload.name
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; // Graceful error mapping

    Ok((StatusCode::CREATED, Json(item)))
}

// GET handler to list all items
async fn get_items(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Item>>, StatusCode> {
    
    let items = sqlx::query_as!(
        Item,
        "SELECT id, name, is_checked FROM items ORDER BY id DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(items))
}