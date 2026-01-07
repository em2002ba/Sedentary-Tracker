use crate::models::ProcessedState;
use sqlx::PgPool;
use tokio::sync::broadcast;

pub async fn spawn_db_worker(pool: PgPool, mut rx: broadcast::Receiver<String>) {
    tokio::spawn(async move {
        println!("Logic Logger Started...");

        while let Ok(json_msg) = rx.recv().await {
            // We deserialize the PROCESSED output, not the raw input
            if let Ok(data) = serde_json::from_str::<ProcessedState>(&json_msg) {
                // Save to 'sedentary_log'
                // We use valid data derived from our Logic Engine
                let result = sqlx::query!(
                    r#"
                    INSERT INTO sedentary_log (state, timer_seconds, acceleration_val)
                    VALUES ($1, $2, $3)
                    "#,
                    data.state,
                    data.timer as i32,
                    data.val
                )
                .execute(&pool)
                .await;

                if let Err(e) = result {
                    eprintln!("DB Error: {}", e);
                }
            }
        }
    });
}
