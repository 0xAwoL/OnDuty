use backend::{server, UsersMap};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let users: UsersMap = Arc::new(RwLock::new(HashMap::new()));

    let users_for_server = Arc::clone(&users);
    let server_handle = tokio::spawn(async move {
        server::run(users_for_server).await;
    });

    tokio::try_join!(server_handle).unwrap();
}
