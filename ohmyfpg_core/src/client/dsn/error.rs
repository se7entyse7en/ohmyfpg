use std::{error, fmt};

#[derive(Debug)]
pub enum InvalidDsnError {
    InvalidDriver(String),
    MissingUser,
    MissingNetloc,
    MissingUserAndNetloc,
    ParseError(String),
}

impl error::Error for InvalidDsnError {}

impl fmt::Display for InvalidDsnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidDsnError::InvalidDriver(driver) => write!(f, "Invalid driver: {}", driver),
            InvalidDsnError::MissingUser => write!(f, "Missing user in Dsn"),
            InvalidDsnError::MissingNetloc => write!(f, "Missing netloc in Dsn"),
            InvalidDsnError::MissingUserAndNetloc => write!(f, "Missing user and netloc in Dsn"),
            InvalidDsnError::ParseError(dsn) => write!(f, "Parsing error for Dsn: {}", dsn),
        }
    }
}
