use crate::messages::{Message, StartupMessage};
use regex::Regex;
use std::{error, fmt, io};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const PATTERN: &str = r"^(?P<driver>postgres(ql)?)://(((?P<user>[\w\d]+)?(:(?P<password>[^@/:\s]+))?@)?(?P<netloc>[\w\d]+)(:(?P<port>\d+))?/?(?P<dbname>[\w\d]+)?(\?)?(?P<params>.*))?$";

#[derive(Debug)]
pub struct DSN {
    pub user: String,
    pub address: String,
    pub password: Option<String>,
    pub dbname: Option<String>,
    pub params: Option<String>,
}

#[derive(Debug)]
pub enum InvalidDSNError {
    InvalidDriver(String),
    MissingUser,
    MissingNetloc,
    MissingUserAndNetloc,
    ParseError(String),
}

impl fmt::Display for InvalidDSNError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidDSNError::InvalidDriver(driver) => write!(f, "Invalid driver: {}", driver),
            InvalidDSNError::MissingUser => write!(f, "Missing user in DSN"),
            InvalidDSNError::MissingNetloc => write!(f, "Missing netloc in DSN"),
            InvalidDSNError::MissingUserAndNetloc => write!(f, "Missing user and netloc in DSN"),
            InvalidDSNError::ParseError(dsn) => write!(f, "Parsing error for DSN: {}", dsn),
        }
    }
}

impl error::Error for InvalidDSNError {}

#[derive(Debug)]
pub enum ConnectionError {
    InvalidDSNError(InvalidDSNError),
    IOError(io::Error),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::InvalidDSNError(err) => write!(f, "Invalid DSM: {}", err),
            ConnectionError::IOError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl error::Error for ConnectionError {}

impl From<io::Error> for ConnectionError {
    fn from(err: io::Error) -> Self {
        ConnectionError::IOError(err)
    }
}

impl From<InvalidDSNError> for ConnectionError {
    fn from(err: InvalidDSNError) -> Self {
        ConnectionError::InvalidDSNError(err)
    }
}

pub fn parse_dsn(dsn: &str) -> Result<DSN, InvalidDSNError> {
    let re = Regex::new(PATTERN).unwrap();
    let caps = re.captures(dsn).unwrap();
    match (caps.name("user"), caps.name("netloc")) {
        (Some(user_match), Some(netloc_match)) => {
            let user = user_match.as_str().to_owned();
            let netloc = netloc_match.as_str();
            let address = caps.name("port").map_or(netloc.to_owned(), |port| {
                format!("{}:{}", netloc, port.as_str())
            });
            let password = caps.name("password").map(|v| v.as_str().to_owned());
            let dbname = caps.name("dbname").map(|v| v.as_str().to_owned());
            let params = caps.name("params").map(|v| v.as_str().to_owned());
            let dsn = DSN {
                user,
                address,
                password,
                dbname,
                params,
            };
            println!("DSN: {:?}", dsn);
            Ok(dsn)
        }
        (Some(_), None) => Err(InvalidDSNError::MissingUser),
        (None, Some(_)) => Err(InvalidDSNError::MissingNetloc),
        (None, None) => Err(InvalidDSNError::MissingUserAndNetloc),
    }
}

pub struct Connection {
    _stream: TcpStream,
}

pub async fn connect(dsn: String) -> Result<Connection, ConnectionError> {
    let parsed_dsn = parse_dsn(&dsn)?;
    let address = parsed_dsn.address;
    println!("Connecting to {}...", address);
    let mut stream = TcpStream::connect(address).await?;
    println!("Connected");
    let mut params = vec![("user".to_owned(), parsed_dsn.user)];
    match parsed_dsn.dbname {
        Some(database) => params.push(("database".to_owned(), database)),
        None => (),
    }
    let msg = StartupMessage::new(params);
    stream.write_all(&msg.serialize()).await?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    println!("Test: {:?}", buffer);

    let connection = Connection { _stream: stream };
    Ok(connection)
}
