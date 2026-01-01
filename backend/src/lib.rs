use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, time::SystemTime};
use tokio::sync::RwLock;

pub mod network_monitor;
pub mod server;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveUser {
    pub name: String,
    pub time_active: SystemTime,
    pub last_seen: SystemTime,
}

pub type UsersMap = Arc<RwLock<HashMap<String, ActiveUser>>>;
