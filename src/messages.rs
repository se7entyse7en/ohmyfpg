pub mod authentication;
pub mod startup;
use crate::client::MessageReadError;
use authentication::{
    AuthenticationOk, AuthenticationSASL, AuthenticationSASLContinue, AuthenticationSASLFinal,
};

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
            _ => Err(MessageReadError::UnrecognizedMessage(
                String::from_utf8(type_.to_vec()).unwrap(),
            )),
        }
    }
}
