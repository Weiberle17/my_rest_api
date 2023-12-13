use axum::{
  body::Body,
  extract::{Path},
  http::{Request, StatusCode},
  middleware::{Next, self},
  response::{IntoResponse, Response},
  routing::get,
  Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, MySqlPool};

#[derive(Serialize, Deserialize, FromRow)]
struct User {
  id: i64,
  name: String,
  email: String,
  age: i64,
}

async fn get_users(Extension(pool): Extension<MySqlPool>) -> impl IntoResponse {
  let users: Vec<User> = match sqlx::query_as("Select id, name, email, age from user")
    .fetch_all(&pool)
    .await
  {
    Ok(rows) => rows,
    Err(e) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal server error: {}", e),
      )
        .into_response()
    }
  };

  (StatusCode::OK, Json(users)).into_response()
}

async fn show_user(
  Extension(pool): Extension<MySqlPool>,
  Path(user_id): Path<u64>,
) -> impl IntoResponse {
  let query = format!(
    "Select id, name, email, age from user where id = {}",
    user_id
  );
  let user: User = match sqlx::query_as(&query).fetch_one(&pool).await {
    Ok(user) => user,
    Err(e) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal server error: {}", e),
      )
        .into_response()
    }
  };

  (StatusCode::OK, Json(user)).into_response()
}

async fn logging_middleware(req: Request<Body>, next: Next<Body>) -> Response {
  println!("Received a request to {}", req.uri());
  next.run(req).await
}

#[tokio::main]
async fn main() {
  let database_url = "mariadb://axum:axum@localhost/axum";
  let pool = MySqlPool::connect(&database_url)
    .await
    .expect("Could not connect to the database");
  let app = Router::new()
    .route("/", get(|| async { "Hello, Rust" }))
    .route("/users", get(get_users))
    .layer(Extension(pool.clone()))
    .route("/users/:id", get(show_user))
    .layer(Extension(pool.clone()))
  .layer(middleware::from_fn(logging_middleware));

  println!("Running on http://localhost:3000");
  axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}
