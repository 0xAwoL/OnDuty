use crate::UsersMap;
use tokio::sync::mpsc;

pub async fn monitor(users: UsersMap) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        interval.tick().await;

        let output = tokio::process::Command::new("arp").arg("-a").output().await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut users_guard = users.write().await;

            for (mac, user) in users_guard.iter_mut() {
                if stdout.contains(mac) {
                    user.last_seen = std::time::SystemTime::now();
                }
            }
        }
    }
}

pub async fn update(tx: mpsc::Sender<String>, users: UsersMap) {
    let kick_time_secs = std::env::var("KICK_TIME")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .unwrap_or(300);
    let kick_time = std::time::Duration::from_secs(kick_time_secs);

    loop {
        let users_guard = users.read().await;

        let mut users_to_remove = Vec::<String>::new();
        // Check for users eligible to be kicked
        for (k, v) in users_guard.iter() {
            if let Ok(elapsed) = v.last_seen.elapsed() {
                if elapsed > kick_time {
                    users_to_remove.push(k.to_string());
                    let _ = tx.send(format!("user {}, has been removed", k)).await;
                }
            }
        }
        drop(users_guard);

        remove_users(users.clone(), users_to_remove).await;
    }
}

async fn remove_users(active_users: UsersMap, users_to_remove: Vec<String>) {
    let mut user = active_users.write().await;
    for k in users_to_remove.iter() {
        user.remove_entry(k);
    }
    drop(user);
}
