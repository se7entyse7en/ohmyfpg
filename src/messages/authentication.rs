#[cfg(test)]
mod tests;
use crate::messages::{DeserializeMessage, SerializeMessage, SerializeMessageBytes};
use base64;
use getrandom::getrandom;

pub const AUTH_MESSAGE_TYPE: &[u8; 1] = b"R";
const SASL_FE_MESSAGE_TYPE: &[u8; 1] = b"p";

// References:
// - https://www.postgresql.org/docs/current/sasl-authentication.html
// - https://github.com/MagicStack/asyncpg/blob/075114c195e9eb4e81c8365d81540beefb46065c/asyncpg/protocol/scram.pyx
// - https://www.2ndquadrant.com/en/blog/password-authentication-methods-in-postgresql/

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
    pub nonce: String,
    pub salt: String,
    pub iteration: usize,
}

impl AuthenticationSASLContinue {
    pub fn new(nonce: String, salt: String, iteration: usize) -> Self {
        AuthenticationSASLContinue {
            nonce,
            salt,
            iteration,
        }
    }
}

impl DeserializeMessage for AuthenticationSASLContinue {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let challenge = String::from_utf8(body[4..].to_vec()).unwrap();
        let fields: Vec<String> = challenge.split(',').map(|s| s.to_owned()).collect();
        let nonce = fields[0][2..].to_owned();
        let salt = fields[1][2..].to_owned();
        let iteration = fields[2][2..].to_owned().parse::<usize>().unwrap();

        AuthenticationSASLContinue::new(nonce, salt, iteration)
    }
}

#[derive(Debug)]
pub struct SASLInitialResponse {
    pub mechanism: String,
    pub user: String,
}

impl SASLInitialResponse {
    pub fn new(mechanism: String, user: String) -> Self {
        SASLInitialResponse { mechanism, user }
    }

    fn create_client_first_message(&self) -> Vec<u8> {
        let mut client_first_message = b"n,,".to_vec();
        let mut data = b"n=".to_vec();
        data.append(&mut self.user.as_bytes().to_vec());
        data.append(&mut b",r=".to_vec());
        data.append(&mut self.generate_client_nonce());
        client_first_message.append(&mut data);
        client_first_message
    }

    fn generate_client_nonce(&self) -> Vec<u8> {
        let mut nonce = [0u8; 16];
        getrandom(&mut nonce).unwrap();
        let mut encoded_nonce = Vec::new();
        // See: https://docs.rs/base64/latest/base64/fn.encode_config_slice.html#example
        encoded_nonce.resize(nonce.len() * 4 / 3 + 4, 0);
        let bytes_written =
            base64::encode_config_slice(nonce, base64::STANDARD, &mut encoded_nonce);
        encoded_nonce.resize(bytes_written, 0);
        encoded_nonce
    }
}

impl SerializeMessage for SASLInitialResponse {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(SASL_FE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = self.mechanism.to_owned().to_msg_bytes();
        let mut client_first_message = self.create_client_first_message();
        let client_first_message_count: u32 = client_first_message.len().try_into().unwrap();
        body.append(&mut client_first_message_count.to_msg_bytes());
        body.append(&mut client_first_message);
        body
    }
}

#[derive(Debug)]
pub struct SASLResponse {
    pub nonce: String,
    pub password: String,
}

impl SASLResponse {
    pub fn new(nonce: String, password: String) -> Self {
        SASLResponse { nonce, password }
    }

    fn create_client_final_message(&self) -> Vec<u8> {
        let mut client_final_message = b"c=".to_vec();
        let channel_binding = b"n,,";
        let mut encoded_channel_binding = Vec::new();
        // See: https://docs.rs/base64/latest/base64/fn.encode_config_slice.html#example
        encoded_channel_binding.resize(channel_binding.len() * 4 / 3 + 4, 0);
        let bytes_written =
            base64::encode_config_slice(b"n,,", base64::STANDARD, &mut encoded_channel_binding);
        encoded_channel_binding.resize(bytes_written, 0);
        client_final_message.append(&mut encoded_channel_binding);
        let mut data = b",r=".to_vec();
        data.append(&mut self.nonce.as_bytes().to_vec());
        data.append(&mut b",p=".to_vec());
        data.append(&mut self.generate_hashed_password());
        client_final_message.append(&mut data);
        println!("client-final-message: {:?}", client_final_message);
        println!(
            "client-final-message (STRING): {:?}",
            String::from_utf8(client_final_message.to_owned()).unwrap()
        );
        client_final_message
    }

    fn generate_hashed_password(&self) -> Vec<u8> {
        // TODO: Handle case with prohibited byte sequences according to `SASLprep`
        Vec::new()
    }
}

impl SerializeMessage for SASLResponse {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(SASL_FE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        self.create_client_final_message()
    }
}
