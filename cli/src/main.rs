use crate::error::CliError;
use clap::{Parser, Subcommand};
use std::{
    io::{Read, Write},
    net::TcpStream,
};
use tracking::{Elements, Observer};
mod error;

#[derive(Parser, Debug)]
#[command(version, about = "Ground Station CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands
/// - get-elements
/// - set-elements <elements>
/// - get-observer
/// - set-observer <latitude> <longitude> <altitude>
///
///  Elements format: name (first line), line1 (second line), line2 (third line)
#[derive(Subcommand, Debug)]
enum Commands {
    /// Get current Elements
    #[command(name = "get-elements")]
    GetElements,

    /// Set Elements with complete Elements data (name and two lines)
    #[command(name = "set-elements")]
    SetElements {
        /// Elements data (name and two lines)
        elements: String,
    },

    /// Get current observer coordinates
    #[command(name = "get-observer")]
    GetObserver,

    /// Set observer coordinates
    #[command(name = "set-observer")]
    SetObserver {
        /// Latitude in degrees
        latitude: f64,
        /// Longitude in degrees
        longitude: f64,
        /// Altitude in meters
        altitude: f64,
    },
    #[command(name = "ping")]
    Ping,
}

/// Parse Elements from a string containing name and two lines
fn parse_elements(elements: &str) -> Result<Elements, CliError> {
    let lines: Vec<&str> = elements.lines().collect();

    if lines.len() < 3 {
        return Err(CliError::InvalidElementsFormat);
    }

    let name = lines[0].trim().to_string();
    let line1 = lines[1].trim().to_string();
    let line2 = lines[2].trim().to_string();

    let e = Elements::from_tle(Some(name), line1.as_bytes(), line2.as_bytes())
        .map_err(|_| CliError::ElementsParseError)?;

    Ok(e)
}

/// Execute the given command and return the command string to send to the ground station
fn execute_command(command: &Commands) -> Result<String, CliError> {
    match command {
        Commands::GetElements => {
            println!("Getting current Elements from ground station...");
            Ok("GET_ELEMENTS".to_string())
        }
        Commands::SetElements { elements } => match parse_elements(elements) {
            Ok(element) => {
                println!("Setting Elements on ground station...");

                let element_json =
                    serde_json::to_string(&element).map_err(|_| CliError::SerializationError)?;
                Ok(format!("SET_ELEMENTS={}", element_json))
            }
            Err(e) => Err(e),
        },
        Commands::GetObserver => {
            println!("Getting current observer from ground station...");
            Ok("GET_OBSERVER".to_string())
        }
        Commands::SetObserver {
            latitude,
            longitude,
            altitude,
        } => {
            println!("Setting observer on ground station...");

            let obs = Observer::new(*latitude, *longitude, *altitude);
            let obs_json = serde_json::to_string(&obs).map_err(|_| CliError::SerializationError)?;
            Ok(format!("SET_OBSERVER={}", obs_json))
        }
        Commands::Ping => {
            println!("Pinging ground station...");
            Ok("PING".to_string())
        }
    }
}

fn main() {
    let args = Args::parse();

    let command = match execute_command(&args.command) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match TcpStream::connect(("localhost", 9999)) {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(command.as_bytes()) {
                eprintln!("Error sending command: {}", e);
                std::process::exit(1);
            }

            let mut response = String::new();
            match stream.read_to_string(&mut response) {
                Ok(_) => {
                    let response = response.trim();
                    if !response.is_empty() {
                        println!("{}", response);
                    } else {
                        println!("Command sent successfully");
                    }
                }
                Err(e) => {
                    eprintln!("Error reading response: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error connecting to ground station: {}", e);
            std::process::exit(1);
        }
    }
}
