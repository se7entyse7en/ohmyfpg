use crate::messages::RawBackendMessage;
use std::{error, fmt};

#[derive(Debug)]
pub struct UnrecognizedMessageError {
    pub raw_message: RawBackendMessage,
}

impl error::Error for UnrecognizedMessageError {}

impl fmt::Display for UnrecognizedMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unrecognized message error: {:?}", self.raw_message)
    }
}

impl UnrecognizedMessageError {
    pub fn new(raw_message: RawBackendMessage) -> Self {
        UnrecognizedMessageError { raw_message }
    }
}
