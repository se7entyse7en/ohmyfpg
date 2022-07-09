use crate::messages::authentication::SASLInitialResponse;
use crate::messages::startup::StartupMessage;
use crate::messages::{BackendMessage, SerializeMessage};
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
    UnrecognizedMessage(String),
    IOError(io::Error),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::InvalidDSNError(err) => write!(f, "Invalid DSM: {}", err),
            ConnectionError::IOError(err) => write!(f, "IO Error: {}", err),
            ConnectionError::UnrecognizedMessage(msg_type) => {
                write!(f, "Unrecognized message type: {}", msg_type)
            }
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

impl From<MessageReadError> for ConnectionError {
    fn from(err: MessageReadError) -> Self {
        match err {
            MessageReadError::UnrecognizedMessage(msg_type) => {
                ConnectionError::UnrecognizedMessage(msg_type)
            }
            MessageReadError::IOError(err) => ConnectionError::IOError(err),
        }
    }
}

#[derive(Debug)]
pub enum MessageReadError {
    UnrecognizedMessage(String),
    IOError(io::Error),
}

impl From<io::Error> for MessageReadError {
    fn from(err: io::Error) -> Self {
        MessageReadError::IOError(err)
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
    stream: TcpStream,
}

impl Connection {
    pub async fn write_message<T>(&mut self, msg: T) -> Result<(), io::Error>
    where
        T: SerializeMessage,
    {
        self.stream.write_all(&msg.serialize()).await
    }

    pub async fn read_message(&mut self) -> Result<BackendMessage, MessageReadError> {
        let mut header = [0u8; 5];
        self.stream.read_exact(&mut header).await?;
        let type_: [u8; 1] = header[0..1].try_into().unwrap();
        let count: [u8; 4] = header[1..5].try_into().unwrap();
        let mut body = vec![0u8; (u32::from_be_bytes(count) - 4).try_into().unwrap()];
        self.stream.read_exact(&mut body).await?;
        BackendMessage::parse(type_, count, body)
    }
}

pub async fn connect(dsn: String) -> Result<Connection, ConnectionError> {
    let parsed_dsn = parse_dsn(&dsn)?;
    let address = parsed_dsn.address;
    println!("Connecting to {}...", address);
    let stream = TcpStream::connect(address).await?;
    let mut connection = Connection { stream };
    println!("Connected");
    // TODO: Avoid this copy, usage slices when possible
    let user = parsed_dsn.user.to_owned();
    let mut params = vec![("user".to_owned(), parsed_dsn.user)];
    match parsed_dsn.dbname {
        Some(database) => params.push(("database".to_owned(), database)),
        None => (),
    }
    let msg = StartupMessage::new(params);
    connection.write_message(msg).await?;
    let msg = connection.read_message().await?;
    println!("Message: {:?}", msg);

    match msg {
        BackendMessage::AuthenticationSASL(msg) => {
            let mechanism = msg.mechanisms[0].to_owned();
            let next_msg = SASLInitialResponse::new(mechanism, user);
            connection.write_message(next_msg).await?;
        }
        _ => todo!("Non-SASL auth"),
    }

    let msg_back = connection.read_message().await?;
    println!("Message back: {:?}", msg_back);

    Ok(connection)
}
