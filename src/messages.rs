pub mod authentication;
pub mod query;
pub mod startup;
use crate::client::MessageReadError;
use authentication::{
    AuthenticationOk, AuthenticationSASL, AuthenticationSASLContinue, AuthenticationSASLFinal,
};
use query::{DataRow, RowDescription};
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

impl BackendMessage {
    pub fn parse(type_: [u8; 1], count: [u8; 4], body: Vec<u8>) -> Result<Self, MessageReadError> {
        match &type_ {
            authentication::AUTH_MESSAGE_TYPE => {
                let raw_sub_type: [u8; 4] = body[..4].try_into().unwrap();
                let count = u32::from_be_bytes(count);
                let sub_type = u32::from_be_bytes(raw_sub_type);
                match sub_type {
                    10_u32 => Ok(BackendMessage::AuthenticationSASL(
                        AuthenticationSASL::deserialize_body(body),
                    )),
                    11_u32 => Ok(BackendMessage::AuthenticationSASLContinue(
                        AuthenticationSASLContinue::deserialize_body(body),
                    )),
                    12_u32 => Ok(BackendMessage::AuthenticationSASLFinal(
                        AuthenticationSASLFinal::deserialize_body(body),
                    )),
                    0_u32 => Ok(BackendMessage::AuthenticationOk(
                        AuthenticationOk::deserialize_body(body),
                    )),
                    _ => Err(MessageReadError::UnrecognizedMessage(format!(
                        "type={},count={},sub_type={},raw_body={:?}",
                        String::from_utf8(type_.to_vec()).unwrap(),
                        count,
                        sub_type,
                        body,
                    ))),
                }
            }
            PARAMETER_STATUS_TYPE => {
                Ok(BackendMessage::ParameterStatus(ParameterStatus::default()))
            }
            BACKEND_KEY_DATA_TYPE => Ok(BackendMessage::BackendKeyData(BackendKeyData::default())),
            NOTICE_RESPONSE_TYPE => Ok(BackendMessage::NoticeResponse(NoticeResponse::default())),
            READY_FOR_QUERY_TYPE => Ok(BackendMessage::ReadyForQuery(ReadyForQuery::default())),
            ERROR_MESSAGE_TYPE => Ok(BackendMessage::ErrorResponse(
                ErrorResponse::deserialize_body(body),
            )),
            query::ROW_DESCRIPTION_MESSAGE_TYPE => Ok(BackendMessage::RowDescription(
                RowDescription::deserialize_body(body),
            )),
            query::DATA_ROW_MESSAGE_TYPE => {
                Ok(BackendMessage::DataRow(DataRow::deserialize_body(body)))
            }
            _ => Err(MessageReadError::UnrecognizedMessage(
                String::from_utf8(type_.to_vec()).unwrap(),
            )),
        }
    }
}
