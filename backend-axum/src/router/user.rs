use axum::{
    extract::State,
    Json,
    routing::post,
    Router
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{MySqlPool, Row};

// init router
pub fn init_router() -> Router<MySqlPool> {
    let app = Router::new()
        .route("/signin", post(signin))
        .route("/signup", post(signup));
    Router::new()
        .nest("/api", app)
}

// user form
#[derive(Deserialize)]
pub struct UserForm {
    #[serde(default)]
    name: String,
    #[serde(default)]
    password: String
}

// signin
pub async fn signin(
    State(pool): State<MySqlPool>,
    Json(user_form): Json<UserForm>
) -> Json<serde_json::Value> {
    let name = user_form.name.trim();
    let password = user_form.password.trim();

    if name == "" || password == "" {
        return Json(json!({
            "code": 1,
            "msg": "parameters missing"
        }))
    }

    let password_hash = sqlx::query("SELECT password_hash FROM user WHERE name = ?")
        .bind(name)
        .fetch_one(&pool)
        .await;

    let password_hash = match password_hash {
        Ok(password_hash) => {
            let hash_str: Vec<u8> = password_hash.get(0);
            String::from_utf8(hash_str).unwrap()
        },
        Err(sqlx::Error::RowNotFound) => {
            return Json(json!({
                "code": 1,
                "msg": "name or password not correct"
            }))
        },
        Err(err) => {
            tracing::error!("select user error: {}", err.to_string());
            return Json(json!({
                "code": 1,
                "msg": "signin failed"
            }))
        }
    };

    let valid = bcrypt::verify(password, &password_hash).unwrap();
    if valid {
        return Json(json!({
            "code": 0,
            "msg": "signin success"
        }))
    } else {
        return Json(json!({
            "code": 1,
            "msg": "name or password not correct"
        }))
    }
}

// signup
pub async fn signup(
    State(pool): State<MySqlPool>,
    Json(user_form): Json<UserForm>
) -> Json<serde_json::Value> {
    let name = user_form.name.trim();
    let password = user_form.password.trim();

    if name == "" || password == "" {
        return Json(json!({
            "code": 1,
            "msg": "parameters missing"
        }))
    }

    let result = sqlx::query("SELECT id FROM user WHERE name = ?")
        .bind(name)
        .fetch_one(&pool)
        .await;

    match result {
        Ok(_) => {
            return Json(json!({
                "code": 1,
                "msg": "name already exist"
            }))
        },
        Err(sqlx::Error::RowNotFound) => {
            // just skip
        },
        Err(err) => {
            tracing::error!("select user error: {}", err.to_string());
            return Json(json!({
                "code": 1,
                "msg": "signup failed"
            }))
        }
    };

    let password_hash = bcrypt::hash(password, 10).unwrap();
    let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at = created_at.clone();

    let result = sqlx::query("INSERT INTO user(name, password_hash, created_at, updated_at) VALUES(?, ?, ?, ?)")
        .bind(name)
        .bind(password_hash)
        .bind(created_at)
        .bind(updated_at)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => {
            return Json(json!({
                "code": 0,
                "msg": "signup success"
            }))
        },
        Err(err) => {
            tracing::error!("insert user error: {}", err.to_string());
            return Json(json!({
                "code": 1,
                "msg": "signup failed"
            }))
        }
    }
}
