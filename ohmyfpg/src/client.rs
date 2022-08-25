use crate::messages::authentication::sasl_authenticate;
use crate::messages::query::{DataRow, Query, RowDescription};
use crate::messages::startup::StartupMessage;
use crate::messages::{BackendMessage, ErrorResponse, SerializeMessage};
use crate::server::PgType;
use regex::Regex;
use std::collections::HashMap;
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
pub struct ServerError {
    pub severity: String,
    pub code: String,
    pub message: String,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (code: {})",
            self.severity, self.message, self.code
        )
    }
}

impl From<ErrorResponse> for ServerError {
    fn from(err_resp: ErrorResponse) -> Self {
        ServerError {
            severity: err_resp.severity,
            code: err_resp.code,
            message: err_resp.message,
        }
    }
}

impl error::Error for ServerError {}

#[derive(Debug)]
pub enum ConnectionError {
    InvalidDSNError(InvalidDSNError),
    UnrecognizedMessage(String),
    UnexpectedMessage(BackendMessage),
    ServerError(ServerError),
    IOError(io::Error),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::InvalidDSNError(err) => write!(f, "Invalid DSM: {}", err),
            ConnectionError::IOError(err) => write!(f, "IO Error: {}", err),
            ConnectionError::ServerError(err) => write!(f, "{}", err),
            ConnectionError::UnexpectedMessage(msg) => {
                write!(f, "Unexpected message: {:?}", msg)
            }
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
            MessageReadError::UnexpectedMessage(msg) => ConnectionError::UnexpectedMessage(msg),
            MessageReadError::IOError(err) => ConnectionError::IOError(err),
        }
    }
}

#[derive(Debug)]
pub enum MessageReadError {
    UnrecognizedMessage(String),
    UnexpectedMessage(BackendMessage),
    IOError(io::Error),
}

impl From<io::Error> for MessageReadError {
    fn from(err: io::Error) -> Self {
        MessageReadError::IOError(err)
    }
}

impl fmt::Display for MessageReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageReadError::UnrecognizedMessage(msg_type) => {
                write!(f, "Unrecognized message type: {}", msg_type)
            }
            MessageReadError::UnexpectedMessage(msg) => {
                write!(f, "Unexpected message: {:?}", msg)
            }
            MessageReadError::IOError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl error::Error for MessageReadError {}

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
            Ok(dsn)
        }
        (Some(_), None) => Err(InvalidDSNError::MissingUser),
        (None, Some(_)) => Err(InvalidDSNError::MissingNetloc),
        (None, None) => Err(InvalidDSNError::MissingUserAndNetloc),
    }
}

pub type FetchResult = HashMap<String, ColumnResult>;

#[derive(Debug)]
pub struct ColumnResult {
    pub bytes: Vec<u8>,
    pub dtype: String,
}

impl ColumnResult {
    pub fn new(bytes: Vec<u8>, dtype: String) -> Self {
        Self { bytes, dtype }
    }
}

pub struct Connection {
    stream: TcpStream,
    pg_types: Option<HashMap<u32, PgType>>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            pg_types: None,
        }
    }

    pub async fn write_message<T>(&mut self, msg: T) -> Result<(), io::Error>
    where
        T: SerializeMessage + fmt::Debug,
    {
        println!("-> Sending message: {:?}", msg);
        self.stream.write_all(&msg.serialize()).await
    }

    pub async fn read_message(&mut self) -> Result<BackendMessage, MessageReadError> {
        let mut header = [0u8; 5];
        self.stream.read_exact(&mut header).await?;
        let type_: [u8; 1] = header[0..1].try_into().unwrap();
        let count: [u8; 4] = header[1..5].try_into().unwrap();
        let mut body = vec![0u8; (u32::from_be_bytes(count) - 4).try_into().unwrap()];
        self.stream.read_exact(&mut body).await?;
        println!("<- Received raw message: {:?}", &body);
        let resp = BackendMessage::parse(type_, count, body);
        println!("<- Received message: {:?}", resp);
        resp
    }

    pub async fn fetch_raw(
        &mut self,
        query_string: String,
    ) -> Result<(RowDescription, Vec<DataRow>), MessageReadError> {
        self.write_message(Query::new(query_string)).await?;

        match self.read_message().await? {
            BackendMessage::RowDescription(row_desc) => {
                let mut data_rows = Vec::with_capacity(row_desc.fields.len());
                let mut error: Option<MessageReadError> = None;
                loop {
                    match self.read_message().await? {
                        BackendMessage::DataRow(data_row) => data_rows.push(data_row),
                        BackendMessage::ReadyForQuery(_) => {
                            break;
                        }
                        BackendMessage::CommandComplete(_) => {}
                        msg => {
                            error = Some(MessageReadError::UnexpectedMessage(msg));
                            break;
                        }
                    };
                }

                match error {
                    Some(err) => Err(err),
                    None => Ok((row_desc, data_rows)),
                }
            }
            msg => Err(MessageReadError::UnexpectedMessage(msg)),
        }
    }

    pub async fn fetch(&mut self, query_string: String) -> Result<FetchResult, MessageReadError> {
        let mut fr = FetchResult::new();
        let (desc, rows) = self.fetch_raw(query_string).await?;
        let mut cols_meta = vec![];
        for field in desc.fields {
            let field_name = field.name;
            let bytes = Vec::with_capacity(rows.len());
            let pg_type = self
                .pg_types
                .as_ref()
                .unwrap()
                .get(&field.data_type_oid)
                .unwrap();
            let pg_type_name = pg_type.name.as_str();
            let dtype_prefix = match pg_type_name {
                "int2" | "int4" | "int8" => "i".to_owned(),
                "float4" | "float8" => "f".to_owned(),
                _ => todo!("{}", format!("Unsopported pg_type: {pg_type_name}")),
            };
            let dtype = format!(">{}{}", dtype_prefix, pg_type.size.unwrap());

            cols_meta.push((field_name.to_owned(), pg_type_name));
            fr.insert(field_name.to_owned(), ColumnResult::new(bytes, dtype));
        }

        for r in rows {
            for (i, c) in r.columns.into_iter().enumerate() {
                // TODO: handle `null`s
                let raw_str_value = String::from_utf8(c.unwrap()).unwrap();

                match cols_meta[i].1 {
                    "int2" => {
                        let bytes = raw_str_value.parse::<i16>().unwrap().to_be_bytes();
                        let col_res = fr.get_mut(&cols_meta[i].0).unwrap();
                        col_res.bytes.extend_from_slice(&bytes);
                    }
                    "int4" => {
                        let bytes = raw_str_value.parse::<i32>().unwrap().to_be_bytes();
                        let col_res = fr.get_mut(&cols_meta[i].0).unwrap();
                        col_res.bytes.extend_from_slice(&bytes);
                    }
                    "int8" => {
                        let bytes = raw_str_value.parse::<i64>().unwrap().to_be_bytes();
                        let col_res = fr.get_mut(&cols_meta[i].0).unwrap();
                        col_res.bytes.extend_from_slice(&bytes);
                    }
                    "float4" => {
                        let bytes = raw_str_value.parse::<f32>().unwrap().to_be_bytes();
                        let col_res = fr.get_mut(&cols_meta[i].0).unwrap();
                        col_res.bytes.extend_from_slice(&bytes);
                    }
                    "float8" => {
                        let bytes = raw_str_value.parse::<f64>().unwrap().to_be_bytes();
                        let col_res = fr.get_mut(&cols_meta[i].0).unwrap();
                        col_res.bytes.extend_from_slice(&bytes);
                    }
                    _ => todo!("{}", format!("Unsopported pg_type: {}", cols_meta[i].1)),
                };
            }
        }
        Ok(fr)
    }
}

pub async fn connect(dsn: String) -> Result<Connection, ConnectionError> {
    let parsed_dsn = parse_dsn(&dsn)?;
    let address = parsed_dsn.address;
    println!("Connecting to {}...", address);
    let stream = TcpStream::connect(address).await?;
    let mut connection = Connection::new(stream);
    println!("Connected!");
    let mut params = vec![("user".to_owned(), parsed_dsn.user.to_owned())];
    match parsed_dsn.dbname {
        Some(database) => params.push(("database".to_owned(), database)),
        None => (),
    }
    let startup = StartupMessage::new(params);

    connection.write_message(startup).await?;
    match connection.read_message().await? {
        BackendMessage::AuthenticationSASL(auth_sasl) => {
            let password = parsed_dsn.password.unwrap();
            sasl_authenticate(&mut connection, &parsed_dsn.user, &password, auth_sasl).await?;
        }
        _ => todo!("Non-SASL auth"),
    }

    loop {
        if let BackendMessage::ReadyForQuery(_) = connection.read_message().await? {
            break;
        }
    }

    println!("Fetching PG types...");
    let query_string = r#"
SELECT oid, typname, typlen
FROM pg_type
WHERE typname IN (
  'int2', 'int4', 'int8', 'numeric', 'float4', 'float8'
);
"#
    .to_owned();
    let (_, data_rows) = connection.fetch_raw(query_string).await?;
    let mut pg_types = HashMap::new();
    for mut dr in data_rows.into_iter() {
        let raw_oid = dr.columns[0].take().unwrap();
        let raw_name = dr.columns[1].take().unwrap();
        let raw_size = dr.columns[2].take().unwrap();

        let s_oid = String::from_utf8(raw_oid).unwrap();
        let name = String::from_utf8(raw_name).unwrap();
        let s_size = String::from_utf8(raw_size).unwrap();

        let oid: u32 = s_oid.parse().unwrap();
        let size: Option<u8> = match s_size.as_str() {
            "-1" => None,
            s => Some(s.parse().unwrap()),
        };

        pg_types.insert(oid, PgType::new(oid, name, size));
    }
    connection.pg_types = Some(pg_types);
    println!("PG types: {:?}", connection.pg_types);
    println!("PG types fetched!");
    Ok(connection)
}
