use std::fmt;

/// Custom error type for CLI operations
#[derive(Debug)]
pub enum CliError {
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
