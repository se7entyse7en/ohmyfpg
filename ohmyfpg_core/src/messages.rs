pub mod authentication;
pub mod error;
pub use error::UnrecognizedMessageError;
pub mod query;
pub mod startup;
use authentication::{
    AuthenticationOk, AuthenticationSASL, AuthenticationSASLContinue, AuthenticationSASLFinal,
};
use query::{BindComplete, CommandComplete, DataRow, ParseComplete, RowDescription};
use std;
use std::collections::HashMap;

const ERROR_MESSAGE_TYPE: &[u8; 1] = b"E";
const PARAMETER_STATUS_TYPE: &[u8; 1] = b"S";
const BACKEND_KEY_DATA_TYPE: &[u8; 1] = b"K";
const NOTICE_RESPONSE_TYPE: &[u8; 1] = b"N";
const READY_FOR_QUERY_TYPE: &[u8; 1] = b"Z";

pub trait SerializeMessage: Sized {
    fn serialize(self) -> Vec<u8> {
        let mut ser = self.serialize_msg_type().unwrap_or_default();
        let mut body = self.serialize_body();
        let count: u32 = (4 + body.len()).try_into().unwrap();
        ser.append(&mut count.to_msg_bytes());
        ser.append(&mut body);

        ser
    }

    fn serialize_msg_type(&self) -> Option<Vec<u8>> {
        self.get_msg_type().map(|s| s.to_owned().to_vec())
    }

    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        None
    }

    fn serialize_body(self) -> Vec<u8>;
}

pub trait DeserializeMessage {
    fn deserialize_body(body: Vec<u8>) -> Self;
}

pub trait SerializeMessageBytes {
    fn to_msg_bytes(self) -> Vec<u8>;
}

impl SerializeMessageBytes for String {
    fn to_msg_bytes(self) -> Vec<u8> {
        let mut ser = self.into_bytes();
        ser.push(0x00);
        ser
    }
}

impl SerializeMessageBytes for u16 {
    fn to_msg_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl SerializeMessageBytes for u32 {
    fn to_msg_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

#[derive(Debug)]
pub struct RawBackendMessage {
    type_: [u8; 1],
    body: Vec<u8>,
}

impl RawBackendMessage {
    pub fn new(type_: [u8; 1], body: Vec<u8>) -> Self {
        RawBackendMessage { type_, body }
    }

    pub fn identify(self) -> Result<RawTypedBackendMessage, error::UnrecognizedMessageError> {
        match &self.type_ {
            authentication::AUTH_MESSAGE_TYPE => {
                let raw_sub_type: [u8; 4] = self.body[..4].try_into().unwrap();
                let sub_type = u32::from_be_bytes(raw_sub_type);
                match sub_type {
                    10_u32 => Ok(RawTypedBackendMessage::AuthenticationSASL(self.body)),
                    11_u32 => Ok(RawTypedBackendMessage::AuthenticationSASLContinue(
                        self.body,
                    )),
                    12_u32 => Ok(RawTypedBackendMessage::AuthenticationSASLFinal(self.body)),
                    0_u32 => Ok(RawTypedBackendMessage::AuthenticationOk(self.body)),
                    _ => Err(error::UnrecognizedMessageError::new(self)),
                }
            }
            PARAMETER_STATUS_TYPE => Ok(RawTypedBackendMessage::ParameterStatus(self.body)),
            BACKEND_KEY_DATA_TYPE => Ok(RawTypedBackendMessage::BackendKeyData(self.body)),
            NOTICE_RESPONSE_TYPE => Ok(RawTypedBackendMessage::NoticeResponse(self.body)),
            READY_FOR_QUERY_TYPE => Ok(RawTypedBackendMessage::ReadyForQuery(self.body)),
            ERROR_MESSAGE_TYPE => Ok(RawTypedBackendMessage::ErrorResponse(self.body)),
            query::ROW_DESCRIPTION_MESSAGE_TYPE => {
                Ok(RawTypedBackendMessage::RowDescription(self.body))
            }
            query::DATA_ROW_MESSAGE_TYPE => Ok(RawTypedBackendMessage::DataRow(self.body)),
            query::COMMAND_COMPLETE_MESSAGE_TYPE => {
                Ok(RawTypedBackendMessage::CommandComplete(self.body))
            }
            query::PARSE_COMPLETE_MESSAGE_TYPE => {
                Ok(RawTypedBackendMessage::ParseComplete(self.body))
            }
            query::BIND_COMPLETE_MESSAGE_TYPE => {
                Ok(RawTypedBackendMessage::BindComplete(self.body))
            }
            _ => Err(error::UnrecognizedMessageError::new(self)),
        }
    }

    pub fn parse(self) -> Result<BackendMessage, error::UnrecognizedMessageError> {
        Ok(self.identify()?.parse())
    }
}

#[derive(Debug)]
pub enum RawTypedBackendMessage {
    AuthenticationSASL(Vec<u8>),
    AuthenticationSASLContinue(Vec<u8>),
    AuthenticationSASLFinal(Vec<u8>),
    AuthenticationOk(Vec<u8>),
    ErrorResponse(Vec<u8>),
    ParameterStatus(Vec<u8>),
    BackendKeyData(Vec<u8>),
    NoticeResponse(Vec<u8>),
    ReadyForQuery(Vec<u8>),
    RowDescription(Vec<u8>),
    DataRow(Vec<u8>),
    CommandComplete(Vec<u8>),
    ParseComplete(Vec<u8>),
    BindComplete(Vec<u8>),
}

impl RawTypedBackendMessage {
    pub fn parse(self) -> BackendMessage {
        match self {
            RawTypedBackendMessage::AuthenticationSASL(body) => {
                BackendMessage::AuthenticationSASL(AuthenticationSASL::deserialize_body(body))
            }
            RawTypedBackendMessage::AuthenticationSASLContinue(body) => {
                BackendMessage::AuthenticationSASLContinue(
                    AuthenticationSASLContinue::deserialize_body(body),
                )
            }
            RawTypedBackendMessage::AuthenticationSASLFinal(body) => {
                BackendMessage::AuthenticationSASLFinal(AuthenticationSASLFinal::deserialize_body(
                    body,
                ))
            }
            RawTypedBackendMessage::AuthenticationOk(body) => {
                BackendMessage::AuthenticationOk(AuthenticationOk::deserialize_body(body))
            }

            RawTypedBackendMessage::ErrorResponse(body) => {
                BackendMessage::ErrorResponse(ErrorResponse::deserialize_body(body))
            }
            RawTypedBackendMessage::ParameterStatus(_) => {
                BackendMessage::ParameterStatus(ParameterStatus::default())
            }
            RawTypedBackendMessage::BackendKeyData(_) => {
                BackendMessage::BackendKeyData(BackendKeyData::default())
            }
            RawTypedBackendMessage::NoticeResponse(_) => {
                BackendMessage::NoticeResponse(NoticeResponse::default())
            }
            RawTypedBackendMessage::ReadyForQuery(_) => {
                BackendMessage::ReadyForQuery(ReadyForQuery::default())
            }
            RawTypedBackendMessage::RowDescription(body) => {
                BackendMessage::RowDescription(RowDescription::deserialize_body(body))
            }
            RawTypedBackendMessage::DataRow(body) => {
                BackendMessage::DataRow(DataRow::deserialize_body(body))
            }
            RawTypedBackendMessage::CommandComplete(body) => {
                BackendMessage::CommandComplete(CommandComplete::deserialize_body(body))
            }
            RawTypedBackendMessage::ParseComplete(body) => {
                BackendMessage::ParseComplete(ParseComplete::deserialize_body(body))
            }
            RawTypedBackendMessage::BindComplete(body) => {
                BackendMessage::BindComplete(BindComplete::deserialize_body(body))
            }
        }
    }
}

#[derive(Debug)]
pub enum BackendMessage {
    AuthenticationSASL(AuthenticationSASL),
    AuthenticationSASLContinue(AuthenticationSASLContinue),
    AuthenticationSASLFinal(AuthenticationSASLFinal),
    AuthenticationOk(AuthenticationOk),
    ErrorResponse(ErrorResponse),
    ParameterStatus(ParameterStatus),
    BackendKeyData(BackendKeyData),
    NoticeResponse(NoticeResponse),
    ReadyForQuery(ReadyForQuery),
    RowDescription(RowDescription),
    DataRow(DataRow),
    CommandComplete(CommandComplete),
    ParseComplete(ParseComplete),
    BindComplete(BindComplete),
}

#[derive(Debug)]
pub struct ErrorResponse {
    pub severity: String,
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(severity: String, code: String, message: String) -> Self {
        ErrorResponse {
            severity,
            code,
            message,
        }
    }
}

#[derive(Debug, Default)]
pub struct ParameterStatus {}

#[derive(Debug, Default)]
pub struct BackendKeyData {}

#[derive(Debug, Default)]
pub struct NoticeResponse {}

#[derive(Debug, Default)]
pub struct ReadyForQuery {}

impl DeserializeMessage for ErrorResponse {
    fn deserialize_body(body: Vec<u8>) -> Self {
        // Reference: https://www.postgresql.org/docs/current/protocol-error-fields.html
        let raw_fields = body.split(|b| *b == 0).filter(|rf| !rf.is_empty());
        let mut fields = HashMap::new();
        for rf in raw_fields {
            let key = String::from_utf8(rf[0..1].to_vec()).unwrap();
            let value = String::from_utf8(rf[1..].to_vec()).unwrap();
            fields.insert(key, value);
        }
        let severity = fields.get("S").unwrap().to_owned();
        let code = fields.get("C").unwrap().to_owned();
        let message = fields.get("M").unwrap().to_owned();
        ErrorResponse::new(severity, code, message)
    }
}
