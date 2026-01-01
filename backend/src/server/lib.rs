use std::collections::{hash_map::Entry, HashMap};
use std::time::SystemTime;

use crate::server::middleware::ValidatedJson;
use crate::ActiveUser;
use crate::UsersMap;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ResponseStatus<T> {
    status: bool,
    data: T,
    timestamp: Option<SystemTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ActiveUserResponse {
    name: String,
    time_active: SystemTime,
}

#[derive(Serialize, Deserialize, Validate)]
struct UserPostPayload {
    #[validate(length(min = 3, message = "Name must be at least 3 characters"))]
    name: String,
    #[validate(length(min = 10, message = "Invalid MAC address length"))]
    mac_address: String,
}

async fn list_users(
    State(users): State<UsersMap>,
) -> Json<ResponseStatus<HashMap<String, ActiveUserResponse>>> {
    serialize_active_users(&users).await
}

async fn claim_device(
    State(users): State<UsersMap>,
    ValidatedJson(user_post_payload): ValidatedJson<UserPostPayload>,
) -> Json<ResponseStatus<String>> {
    let time_now: SystemTime = SystemTime::now();
    let verified_payload = ActiveUser {
        name: user_post_payload.name,
        time_active: time_now,
        last_seen: time_now,
    };
    let mut user_guard = users.write().await;
    let (status, message) = match user_guard.entry(user_post_payload.mac_address) {
        Entry::Occupied(_) => (false, "Device already claimed"),
        Entry::Vacant(e) => {
            e.insert(verified_payload);
            (true, "ok")
        }
    };

    Json(ResponseStatus {
        status: status,
        data: message.to_string(),
        timestamp: Some(time_now),
    })
}

fn create_app(users: UsersMap) -> Router {
    Router::new()
        .route("/list_users/", get(list_users))
        .route("/claim_device", post(claim_device))
        .with_state(users)
}

pub async fn run(users: UsersMap) {
    let app = create_app(users);
    let host = std::env::var("SERVER_URL").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    println!("Server is running on {}:{}", host, port);
    axum::serve(listener, app).await.unwrap();
}

pub async fn run_server_for_test(
    users: UsersMap,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let app = create_app(users);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, server_handle)
}

async fn serialize_active_users(
    users: &UsersMap,
) -> Json<ResponseStatus<HashMap<String, ActiveUserResponse>>> {
    let users_guard = users.read().await;
    let data: HashMap<String, ActiveUserResponse> = users_guard
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                ActiveUserResponse {
                    name: v.name.clone(),
                    time_active: v.time_active,
                },
            )
        })
        .collect();

    Json(ResponseStatus {
        status: true,
        data,
        timestamp: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_claim_device_flow() {
        let users: UsersMap = Arc::new(RwLock::new(HashMap::new()));
        let (addr, _handle) = run_server_for_test(users.clone()).await;
        let client = Client::new();
        let url = format!("http://{}", addr);

        // 1. Claim success
        let resp = client
            .post(format!("{}/claim_device", url))
            .json(&serde_json::json!({
                "name": "test_device",
                "mac_address": "00:11:22:33:44:55"
            }))
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
        let body: ResponseStatus<String> = resp.json().await.unwrap();
        assert!(body.status);
        assert_eq!(body.data, "ok");

        // 2. Duplicate claim
        let resp = client
            .post(format!("{}/claim_device", url))
            .json(&serde_json::json!({
                "name": "test_device_2",
                "mac_address": "00:11:22:33:44:55"
            }))
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
        let body: ResponseStatus<String> = resp.json().await.unwrap();
        assert!(!body.status);
        assert_eq!(body.data, "Device already claimed");

        // 3. Validation error (short name)
        let resp = client
            .post(format!("{}/claim_device", url))
            .json(&serde_json::json!({
                "name": "yo",
                "mac_address": "00:11:22:33:44:55"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400);

        // 4. List users
        let resp = client
            .get(format!("{}/list_users/", url))
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
        let body: ResponseStatus<HashMap<String, ActiveUserResponse>> = resp.json().await.unwrap();
        assert!(body.data.contains_key("00:11:22:33:44:55"));
        // Original name should persist because duplicates are rejected
        assert_eq!(
            body.data.get("00:11:22:33:44:55").unwrap().name,
            "test_device"
        );
    }
}
