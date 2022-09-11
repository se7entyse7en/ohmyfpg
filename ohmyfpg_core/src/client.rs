use crate::messages::authentication::sasl_authenticate;
use crate::messages::query::{DataRow, Query, RowDescription};
use crate::messages::startup::StartupMessage;
use crate::messages::{BackendMessage, RawBackendMessage, SerializeMessage};
use crate::server::PgType;
mod dsn;
use std::collections::HashMap;
use std::fmt;
use tokio::io;
use tokio::net::TcpStream;
pub mod error;
mod framer;
pub use error::{ConnectionError, FetchError, MessageReadError, ServerError};
use framer::Framer;

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
    framer: Framer,
    pg_types: Option<HashMap<u32, PgType>>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            framer: Framer::new(stream),
            pg_types: None,
        }
    }

    pub async fn write_message<T>(&mut self, msg: T) -> io::Result<()>
    where
        T: SerializeMessage + fmt::Debug,
    {
        self.framer.write_frame(msg).await
    }

    pub async fn read_raw_message(&mut self) -> io::Result<RawBackendMessage> {
        Ok(self.framer.read_frame().await.unwrap())
    }

    pub async fn read_message(&mut self) -> Result<BackendMessage, MessageReadError> {
        let raw_message = self.read_raw_message().await?;
        raw_message.parse().map_err(MessageReadError::from)
    }

    pub async fn fetch_raw(
        &mut self,
        query_string: String,
    ) -> Result<(RowDescription, Vec<DataRow>), FetchError> {
        self.write_message(Query::new(query_string)).await?;
        let message = self.read_message().await?;
        match message {
            BackendMessage::RowDescription(row_desc) => {
                let mut data_rows = Vec::with_capacity(row_desc.fields.len());
                let mut error: Option<FetchError> = None;
                loop {
                    let message = self.read_message().await?;
                    match message {
                        BackendMessage::DataRow(data_row) => data_rows.push(data_row),
                        BackendMessage::ReadyForQuery(_) => {
                            break;
                        }
                        BackendMessage::CommandComplete(_) => {}
                        msg => {
                            error = Some(FetchError::UnexpectedMessageError(msg));
                            break;
                        }
                    };
                }

                match error {
                    Some(err) => Err(err),
                    None => Ok((row_desc, data_rows)),
                }
            }
            msg => Err(FetchError::UnexpectedMessageError(msg)),
        }
    }

    pub async fn fetch(&mut self, query_string: String) -> Result<FetchResult, FetchError> {
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

pub async fn connect(raw_dsn: String) -> Result<Connection, ConnectionError> {
    let parsed_dsn = dsn::parse_dsn(&raw_dsn)?;
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
    let message = connection.read_message().await?;
    match message {
        BackendMessage::AuthenticationSASL(auth_sasl) => {
            let password = parsed_dsn.password.unwrap();
            sasl_authenticate(&mut connection, &parsed_dsn.user, &password, auth_sasl).await?;
        }
        _ => todo!("Non-SASL auth"),
    }

    loop {
        let message = connection.read_message().await?;
        if let BackendMessage::ReadyForQuery(_) = message {
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
