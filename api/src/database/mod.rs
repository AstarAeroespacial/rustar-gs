use sqlx::{Any, AnyPool, Pool};
use std::path::Path;

pub async fn create_pool(database_url: &str) -> Result<Pool<Any>, sqlx::Error> {
    // Create data directory if it doesn't exist
    if let Some(path) = Path::new(database_url).parent() {
        std::fs::create_dir_all(path).ok();
    }

    let pool = AnyPool::connect(database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
} 