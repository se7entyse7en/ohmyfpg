use crate::messages::authentication::sasl_authenticate;
use crate::messages::query::{
    Bind, DataRow, Describe, Execute, Flush, Format, Parse, RowDescription, Sync,
};
use crate::messages::startup::StartupMessage;
use crate::messages::DeserializeMessage;
use crate::messages::{
    BackendMessage, RawBackendMessage, RawTypedBackendMessage, SerializeMessage,
};
use crate::server::PgType;
use rayon::prelude::*;
mod dsn;
use std::collections::HashMap;
use std::{fmt, str};
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

    async fn read_raw_message(&mut self) -> io::Result<RawBackendMessage> {
        Ok(self.framer.read_frame().await.unwrap())
    }

    async fn read_raw_typed_message(&mut self) -> Result<RawTypedBackendMessage, MessageReadError> {
        let raw_message = self.read_raw_message().await?;
        raw_message.identify().map_err(MessageReadError::from)
    }

    pub async fn read_message(&mut self) -> Result<BackendMessage, MessageReadError> {
        let raw_typed_message = self.read_raw_typed_message().await?;
        Ok(raw_typed_message.parse())
    }

    async fn execute_prepared_query(
        &mut self,
        query_string: String,
    ) -> Result<RowDescription, FetchError> {
        self.write_message(Parse::new(query_string.to_owned()))
            .await?;
        self.write_message(Bind::new(Format::Binary)).await?;
        self.write_message(Describe::default()).await?;
        self.write_message(Execute::default()).await?;
        self.write_message(Flush::default()).await?;

        let _message = self.read_message().await?;
        // TODO: Ensure it's a `ParseComplete`
        let _message = self.read_message().await?;
        // TODO: Ensure it's a `BindComplete`
        let message = self.read_message().await?;

        match message {
            BackendMessage::RowDescription(row_desc) => Ok(row_desc),
            msg => Err(FetchError::UnexpectedMessageError(msg)),
        }
    }

    pub async fn fetch_raw(
        &mut self,
        query_string: String,
    ) -> Result<(RowDescription, Vec<Vec<u8>>), FetchError> {
        let row_desc = self.execute_prepared_query(query_string).await?;
        let mut data_rows = vec![];
        let mut error: Option<FetchError> = None;
        loop {
            let message = self.read_raw_typed_message().await?;
            match message {
                RawTypedBackendMessage::DataRow(body) => data_rows.push(body),
                RawTypedBackendMessage::ReadyForQuery(_) => {
                    break;
                }
                RawTypedBackendMessage::CommandComplete(_) => {
                    self.write_message(Sync::default()).await?;
                }
                msg => {
                    error = Some(FetchError::UnexpectedMessageError(msg.parse()));
                    break;
                }
            };
        }

        match error {
            Some(err) => Err(err),
            None => Ok((row_desc, data_rows)),
        }
    }

    pub async fn fetch(&mut self, query_string: String) -> Result<FetchResult, FetchError> {
        let (desc, data_rows_bytes) = self.fetch_raw(query_string).await?;
        let total_rows = data_rows_bytes.len();
        let mut cols_meta = vec![];
        let mut index_field_map = HashMap::new();
        for (i, field) in desc.fields.into_iter().enumerate() {
            let field_name = field.name;
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

            cols_meta.push((field_name.to_owned(), pg_type_name, dtype));

            index_field_map.insert(i, field_name.to_owned());
        }

        let chunks = data_rows_bytes
            .into_par_iter()
            .map(DataRow::deserialize_body)
            .fold(
                HashMap::new,
                |mut acc: HashMap<&str, Vec<u8>>, dr: DataRow| {
                    for (i, c) in dr.columns.into_iter().enumerate() {
                        // TODO: handle `null`s
                        let raw_bytes = &c.unwrap();

                        let value = match cols_meta[i].1 {
                            "int2" | "int4" | "int8" | "float4" | "float8" => raw_bytes.to_vec(),
                            _ => todo!("{}", format!("Unsopported pg_type: {}", cols_meta[i].1)),
                        };

                        let field_name = index_field_map.get(&i).unwrap();
                        let entry = acc.entry(field_name);
                        entry.or_default().extend(value);
                    }
                    acc
                },
            )
            .collect::<Vec<HashMap<&str, Vec<u8>>>>();

        let mut fr = FetchResult::new();
        for col_meta in cols_meta {
            let field_name = col_meta.0;
            let dtype = &col_meta.2;
            let mut col_res = ColumnResult::new(Vec::with_capacity(total_rows), dtype.to_string());

            for chunk in &chunks {
                let value = chunk.get(&field_name as &str).unwrap();
                col_res.bytes.extend_from_slice(value);
            }

            fr.insert(field_name.to_owned(), col_res);
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
    if let Some(database) = parsed_dsn.dbname {
        params.push(("database".to_owned(), database))
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
    let (_, data_rows_bytes) = connection.fetch_raw(query_string).await?;
    let mut pg_types = HashMap::new();
    for drb in data_rows_bytes.into_iter() {
        let mut dr = DataRow::deserialize_body(drb);
        let raw_oid = dr.columns[0].take().unwrap();
        let raw_name = dr.columns[1].take().unwrap();
        let raw_size = dr.columns[2].take().unwrap();

        let arr_oid: [u8; 4] = raw_oid.to_vec().try_into().unwrap();
        let oid = u32::from_be_bytes(arr_oid);

        let name = String::from_utf8(raw_name.to_vec()).unwrap();

        let arr_size: [u8; 2] = raw_size.to_vec().try_into().unwrap();
        let size = i16::from_be_bytes(arr_size);

        let size: Option<u8> = match size {
            -1 => None,
            s => Some(s as u8),
        };

        pg_types.insert(oid, PgType::new(oid, name, size));
    }
    connection.pg_types = Some(pg_types);
    println!("PG types: {:?}", connection.pg_types);
    println!("PG types fetched!");
    Ok(connection)
}
