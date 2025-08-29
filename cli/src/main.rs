use clap::{Parser, Subcommand};

/// Ground Station CLI
#[derive(Parser, Debug)]
#[command(version, about = "Ground Station CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get current TLE (Two Line Elements)
    #[command(name = "get-tle")]
    GetTle,

    /// Set TLE with complete TLE data (name and two lines)
    #[command(name = "set-tle")]
    SetTle {
        /// Complete TLE data: name, line1, and line2 separated by newlines
        tle_data: String,
    },

    /// Get current location coordinates
    #[command(name = "get-location")]
    GetLocation,

    /// Set location coordinates
    #[command(name = "set-location")]
    SetLocation {
        /// Latitude in degrees
        latitude: f64,
        /// Longitude in degrees
        longitude: f64,
        /// Altitude in meters (optional)
        #[arg(short, long, default_value_t = 0.0)]
        altitude: f64,
    },
}

fn parse_tle(tle_data: &str) -> Result<(String, String, String), String> {
    let lines: Vec<&str> = tle_data.lines().collect();

    if lines.len() < 3 {
        return Err("TLE data must contain at least 3 lines (name, line1, line2)".to_string());
    }

    let name = lines[0].trim().to_string();
    let line1 = lines[1].trim().to_string();
    let line2 = lines[2].trim().to_string();

    Ok((name, line1, line2))
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::GetTle => {
            println!("Getting current TLE...");
            // Implementar lógica para obtener TLE
        }
        Commands::SetTle { tle_data } => {
            match parse_tle(&tle_data) {
                Ok((name, line1, line2)) => {
                    println!("Setting TLE:");
                    println!("Name: {}", name);
                    println!("Line 1: {}", line1);
                    println!("Line 2: {}", line2);
                    // Implementar lógica para guardar TLE
                }
                Err(e) => {
                    eprintln!("Error parsing TLE: {}", e);
                }
            }
        }
        Commands::GetLocation => {
            println!("Getting current location...");
            // Implementar lógica para obtener ubicación
        }
        Commands::SetLocation {
            latitude,
            longitude,
            altitude,
        } => {
            println!("Setting location:");
            println!("Latitude: {:.6}°", latitude);
            println!("Longitude: {:.6}°", longitude);
            println!("Altitude: {:.1} m", altitude);
            // Implementar lógica para guardar ubicación
        }
    }
}
