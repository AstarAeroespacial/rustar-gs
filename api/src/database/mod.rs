use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;

pub async fn create_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Create data directory if it doesn't exist
    if let Some(path) = Path::new(database_url).parent() {
        std::fs::create_dir_all(path).ok();
    }

    let pool = SqlitePool::connect(database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
} 