use std::{
    io::{Read, Write},
    net::TcpStream,
};

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
    /// Get current TLE
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

fn execute_command(command: &Commands) -> Result<String, String> {
    match command {
        Commands::GetTle => {
            println!("Getting current TLE from ground station...");
            Ok("GET_TLE".to_string())
        }
        Commands::SetTle { tle_data } => match parse_tle(&tle_data) {
            Ok((name, line1, line2)) => {
                println!("Setting TLE on ground station...");
                Ok(format!("SET_TLE|{}|{}|{}", name, line1, line2))
            }
            Err(e) => Err(format!("Error parsing TLE: {}", e)),
        },
        Commands::GetLocation => {
            println!("Getting current location from ground station...");
            Ok("GET_LOCATION".to_string())
        }
        Commands::SetLocation {
            latitude,
            longitude,
            altitude,
        } => {
            println!("Setting location on ground station...");
            Ok(format!(
                "SET_LOCATION|{}|{}|{}",
                latitude, longitude, altitude
            ))
        }
    }
}

fn main() {
    let args = Args::parse();

    let command = match execute_command(&args.command) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    match TcpStream::connect("") {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(command.as_bytes()) {
                eprintln!("Error sending command: {}", e);
                return;
            }

            let mut response = String::new();
            match stream.read_to_string(&mut response) {
                Ok(_) => {
                    let response = response.trim();
                    if !response.is_empty() {
                        println!("Response from ground station: {}", response);
                    } else {
                        println!("Command sent successfully");
                    }
                }
                Err(e) => {
                    eprintln!("Error reading response: {}", e);
                }
            }
        }
        Err(_) => {
            eprintln!("Error connecting to ground station");
            std::process::exit(1);
        }
    }
}
