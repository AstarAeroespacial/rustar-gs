use sqlx::{SqlitePool};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let pool = SqlitePool::connect("sqlite:./.data/telemetry.db").await?;
    
    println!("Seeding database with test telemetry data...");
    
    // Generate some test data for the last 24 hours
    let now = Utc::now();
    let mut current_time = now - Duration::hours(24);
    
    for i in 0..100 {
        let telemetry = (
            Uuid::new_v4().to_string(),
            current_time.timestamp(),
            20.0 + (i as f64 * 0.1), // Varying temperature
            12.0 + (i as f64 * 0.01), // Varying voltage
            1.0 + (i as f64 * 0.005), // Varying current
            50 + (i % 20), // Varying battery level
        );
        
        sqlx::query(
            r#"
            INSERT INTO telemetry (id, timestamp, temperature, voltage, current, battery_level)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(telemetry.0)
        .bind(telemetry.1)
        .bind(telemetry.2)
        .bind(telemetry.3)
        .bind(telemetry.4)
        .bind(telemetry.5)
        .execute(&pool)
        .await?;
        
        current_time += Duration::minutes(15); // 15-minute intervals
    }
    
    println!("Successfully seeded database with 100 telemetry records!");
    Ok(())
} 