#[cfg(test)]
mod tests;

use crate::messages::{Message, MessageBytesSerialize};

static MESSAGE_TYPE: &[u8; 1] = b"R";

#[derive(Debug)]
pub struct AuthenticationSASL {
    pub mechanisms: Vec<String>,
}

impl AuthenticationSASL {
    pub fn new(mechanisms: Vec<String>) -> Self {
        AuthenticationSASL { mechanisms }
    }
}

impl Message for AuthenticationSASL {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(MESSAGE_TYPE)
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
