#[cfg(test)]
mod tests;
use crate::messages::DeserializeMessage;
mod sasl;
pub use sasl::authenticate as sasl_authenticate;
pub use sasl::{
    AuthenticationSASL, AuthenticationSASLContinue, AuthenticationSASLFinal, SASLInitialResponse,
    SASLResponse,
};

pub const AUTH_MESSAGE_TYPE: &[u8; 1] = b"R";

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
