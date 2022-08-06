#[cfg(test)]
mod tests;
use crate::messages::{DeserializeMessage, SerializeMessage, SerializeMessageBytes};

pub const AUTH_MESSAGE_TYPE: &[u8; 1] = b"R";
const SASL_FE_MESSAGE_TYPE: &[u8; 1] = b"p";

// References:
// - https://www.postgresql.org/docs/current/sasl-authentication.html
// - https://github.com/MagicStack/asyncpg/blob/075114c195e9eb4e81c8365d81540beefb46065c/asyncpg/protocol/scram.pyx
// - https://www.2ndquadrant.com/en/blog/password-authentication-methods-in-postgresql/
// - Relevant RFCs:
//   - RFC 3454
//   - RFC 4013
//   - RFC 4422
//   - RFC 5802
//   - RFC 5803
//   - RFC 7677

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
pub struct SASLInitialResponse {
    pub mechanism: String,
    pub client_first: String,
}

impl SASLInitialResponse {
    pub fn new(mechanism: String, client_first: String) -> Self {
        SASLInitialResponse {
            mechanism,
            client_first,
        }
    }
}

impl SerializeMessage for SASLInitialResponse {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(SASL_FE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = self.mechanism.to_owned().to_msg_bytes();
        let client_first_count: u32 = self.client_first.len().try_into().unwrap();
        body.append(&mut client_first_count.to_msg_bytes());
        body.append(&mut self.client_first.into_bytes());
        body
    }
}

#[derive(Debug)]
pub struct SASLResponse {
    pub client_final: String,
}

impl SASLResponse {
    pub fn new(client_final: String) -> Self {
        SASLResponse { client_final }
    }
}

impl SerializeMessage for SASLResponse {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(SASL_FE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        self.client_final.into_bytes()
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

#[derive(Debug)]
pub struct AuthenticationOk {}

impl Default for AuthenticationOk {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthenticationOk {
    pub fn new() -> Self {
        AuthenticationOk {}
    }
}

impl DeserializeMessage for AuthenticationOk {
    fn deserialize_body(_: Vec<u8>) -> Self {
        AuthenticationOk::new()
    }
}
