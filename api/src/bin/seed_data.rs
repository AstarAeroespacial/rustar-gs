use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("API_DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to database
    let pool = PgPool::connect(&database_url).await?;

    println!("Seeding database with test telemetry data...");

    // Generate some test data for the last 24 hours
    let now = Utc::now();
    let mut current_time = now - Duration::hours(24);

    for i in 0..100 {
        let telemetry = (
            Uuid::new_v4().to_string(),
            current_time.timestamp(),
            20.0 + (i as f32 * 0.1),  // Varying temperature
            12.0 + (i as f32 * 0.01), // Varying voltage
            1.0 + (i as f32 * 0.005), // Varying current
            50 + (i % 20),            // Varying battery level
        );

        sqlx::query!(
            r#"
            INSERT INTO telemetry (id, timestamp, temperature, voltage, current, battery_level)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            (telemetry.0),
            (telemetry.1),
            (telemetry.2),
            (telemetry.3),
            (telemetry.4),
            (telemetry.5)
        )
        .execute(&pool)
        .await?;

        current_time += Duration::minutes(15); // 15-minute intervals
    }

    println!("Successfully seeded database with 100 telemetry records!");
    Ok(())
}
