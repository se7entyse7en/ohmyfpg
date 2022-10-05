use crate::messages::{SerializeMessage, SerializeMessageBytes};

const QUERY_MESSAGE_TYPE: &[u8; 1] = b"Q";
const PARSE_MESSAGE_TYPE: &[u8; 1] = b"P";
const FLUSH_MESSAGE_TYPE: &[u8; 1] = b"H";

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

#[derive(Debug)]
pub struct Parse {
    pub query_string: String,
}

impl Parse {
    pub fn new(query_string: String) -> Self {
        Parse { query_string }
    }
}

impl SerializeMessage for Parse {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(PARSE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = vec![];
        let mut prepared_stmt_name = "".to_string().to_msg_bytes();
        body.append(&mut prepared_stmt_name);
        body.append(&mut self.query_string.to_msg_bytes());
        body.append(&mut 0u16.to_msg_bytes());
        body
    }
}

#[derive(Debug)]
pub struct Flush {}

impl Flush {
    fn new() -> Self {
        Flush {}
    }
}

impl Default for Flush {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializeMessage for Flush {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(FLUSH_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        vec![]
    }
}
