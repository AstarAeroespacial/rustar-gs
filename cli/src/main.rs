use std::{
    fmt,
    io::{Read, Write},
    net::TcpStream,
};

use clap::{Parser, Subcommand};
use tracking::{Elements, Observer};

/// Custom error type for CLI operations
#[derive(Debug)]
enum CliError {
    ElementsParseError,
    SerializationError,
    InvalidElementsFormat,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::ElementsParseError => write!(f, "Error parsing Elements"),
            CliError::SerializationError => write!(f, "SError serializing data"),
            CliError::InvalidElementsFormat => write!(f, "Invalid Elements format"),
        }
    }
}

/// Ground Station CLI
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
/// - set-observer <latitude> <longitude> [--altitude <altitude>]
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
        /// Altitude in meters (optional)
        #[arg(short, long, default_value_t = 0.0)]
        altitude: f64,
    },
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
            println!("Setting observre on ground station...");
            let obs = Observer::new(*latitude, *longitude, *altitude);
            let obs_json = serde_json::to_string(&obs).map_err(|_| CliError::SerializationError)?;
            Ok(format!("SET_OBSERVER={}", obs_json))
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

    match TcpStream::connect("") {
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
                        println!("Response from ground station: {}", response);
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
