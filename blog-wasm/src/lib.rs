use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const TOKEN_KEY: &str = "blog_token";
const USER_KEY: &str = "blog_user";

// ============ Data Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsList {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
struct RegisterRequest<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct CreatePostRequest<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdatePostRequest<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: String,
}

// ============ Storage Helpers ============

fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn save_to_storage(key: &str, value: &str) -> Result<(), JsValue> {
    get_storage()
        .ok_or_else(|| JsValue::from_str("localStorage not available"))?
        .set_item(key, value)
        .map_err(|_| JsValue::from_str("Failed to save to localStorage"))
}

fn get_from_storage(key: &str) -> Option<String> {
    get_storage()?.get_item(key).ok()?
}

fn remove_from_storage(key: &str) -> Result<(), JsValue> {
    get_storage()
        .ok_or_else(|| JsValue::from_str("localStorage not available"))?
        .remove_item(key)
        .map_err(|_| JsValue::from_str("Failed to remove from localStorage"))
}

// ============ BlogApp ============

#[wasm_bindgen]
pub struct BlogApp {
    base_url: String,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str) -> Self {
        console_error_panic_hook::set_once();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    fn get_token(&self) -> Option<String> {
        get_from_storage(TOKEN_KEY)
    }

    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.get_token().is_some()
    }

    #[wasm_bindgen]
    pub fn get_current_user(&self) -> Result<JsValue, JsValue> {
        match get_from_storage(USER_KEY) {
            Some(json) => {
                let user: User = serde_json::from_str(&json)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                serde_wasm_bindgen::to_value(&user)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            None => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen]
    pub fn logout(&self) -> Result<(), JsValue> {
        remove_from_storage(TOKEN_KEY)?;
        remove_from_storage(USER_KEY)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<JsValue, JsValue> {
        let body = serde_json::to_string(&RegisterRequest {
            username,
            email,
            password,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = Request::post(&self.url("/auth/register"))
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Registration failed".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        let auth: AuthResponse = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        save_to_storage(TOKEN_KEY, &auth.token)?;
        save_to_storage(
            USER_KEY,
            &serde_json::to_string(&auth.user)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        )?;

        serde_wasm_bindgen::to_value(&auth)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<JsValue, JsValue> {
        let body = serde_json::to_string(&LoginRequest { email, password })
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = Request::post(&self.url("/auth/login"))
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Login failed".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        let auth: AuthResponse = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        save_to_storage(TOKEN_KEY, &auth.token)?;
        save_to_storage(
            USER_KEY,
            &serde_json::to_string(&auth.user)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        )?;

        serde_wasm_bindgen::to_value(&auth)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn load_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<JsValue, JsValue> {
        let url =
            format!("{}?limit={}&offset={}", self.url("/posts"), limit, offset);

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Failed to load posts".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        let posts: PostsList = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&posts)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn get_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let response = Request::get(&self.url(&format!("/posts/{}", id)))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Post not found".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        let post: Post = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&post)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
    ) -> Result<JsValue, JsValue> {
        let token = self
            .get_token()
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let body = serde_json::to_string(&CreatePostRequest { title, content })
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = Request::post(&self.url("/posts"))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Failed to create post".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        let post: Post = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&post)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn update_post(
        &self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<JsValue, JsValue> {
        let token = self
            .get_token()
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let body = serde_json::to_string(&UpdatePostRequest { title, content })
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = Request::put(&self.url(&format!("/posts/{}", id)))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Failed to update post".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        let post: Post = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&post)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn delete_post(&self, id: i64) -> Result<(), JsValue> {
        let token = self
            .get_token()
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let response = Request::delete(&self.url(&format!("/posts/{}", id)))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: "Failed to delete post".to_string(),
            });
            return Err(JsValue::from_str(&error.error));
        }

        Ok(())
    }
}

// ============ Console Logging ============

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
