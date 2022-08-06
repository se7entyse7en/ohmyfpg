#[cfg(test)]
use crate::messages::authentication::AUTH_MESSAGE_TYPE;
use crate::messages::DeserializeMessage;
#[cfg(test)]
use crate::messages::{SerializeMessage, SerializeMessageBytes};

#[derive(Debug)]
pub struct AuthenticationSASL {
    pub mechanisms: Vec<String>,
}

impl AuthenticationSASL {
    pub fn new(mechanisms: Vec<String>) -> Self {
        AuthenticationSASL { mechanisms }
    }
}

#[cfg(test)]
impl SerializeMessage for AuthenticationSASL {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(AUTH_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = 10_u32.to_msg_bytes();
        for mechanism in self.mechanisms.into_iter() {
            body.append(&mut mechanism.to_msg_bytes());
        }

        body.push(0x00);
        body
    }
}

impl DeserializeMessage for AuthenticationSASL {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let mut mechanisms = Vec::new();
        let iter = body[4..]
            .split(|b| *b == 0)
            .filter(|chunk| !chunk.is_empty());
        for chunk in iter {
            mechanisms.push(String::from_utf8(chunk.to_vec()).unwrap());
        }

        AuthenticationSASL::new(mechanisms)
    }
}

#[derive(Debug)]
pub struct AuthenticationSASLContinue {
    pub server_first: String,
}

impl AuthenticationSASLContinue {
    pub fn new(server_first: String) -> Self {
        AuthenticationSASLContinue { server_first }
    }
}

impl DeserializeMessage for AuthenticationSASLContinue {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let server_first = String::from_utf8(body[4..].to_vec()).unwrap();
        AuthenticationSASLContinue::new(server_first)
    }
}

#[derive(Debug)]
pub struct AuthenticationSASLFinal {
    pub server_final: String,
}

impl AuthenticationSASLFinal {
    pub fn new(server_final: String) -> Self {
        AuthenticationSASLFinal { server_final }
    }
}

impl DeserializeMessage for AuthenticationSASLFinal {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let server_final = String::from_utf8(body[4..].to_vec()).unwrap();
        AuthenticationSASLFinal::new(server_final)
    }
}
