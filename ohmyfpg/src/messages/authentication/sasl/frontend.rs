use crate::messages::authentication::sasl::SASL_FE_MESSAGE_TYPE;
use crate::messages::{SerializeMessage, SerializeMessageBytes};

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
