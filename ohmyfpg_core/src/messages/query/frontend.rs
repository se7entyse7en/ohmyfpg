use crate::messages::{SerializeMessage, SerializeMessageBytes};

const QUERY_MESSAGE_TYPE: &[u8; 1] = b"Q";

#[derive(Debug)]
pub struct Query {
    pub query_string: String,
}

impl Query {
    pub fn new(query_string: String) -> Self {
        Query { query_string }
    }
}

impl SerializeMessage for Query {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(QUERY_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        self.query_string.to_msg_bytes()
    }
}
