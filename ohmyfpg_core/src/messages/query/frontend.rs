use crate::messages::{SerializeMessage, SerializeMessageBytes};

const QUERY_MESSAGE_TYPE: &[u8; 1] = b"Q";
const PARSE_MESSAGE_TYPE: &[u8; 1] = b"P";
const FLUSH_MESSAGE_TYPE: &[u8; 1] = b"H";
const BIND_MESSAGE_TYPE: &[u8; 1] = b"B";
const DESCRIBE_MESSAGE_TYPE: &[u8; 1] = b"D";
const EXECUTE_MESSAGE_TYPE: &[u8; 1] = b"E";
const SYNC_MESSAGE_TYPE: &[u8; 1] = b"S";

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

#[derive(Debug)]
pub enum Format {
    Text,
    Binary,
}

#[derive(Debug)]
pub struct Bind {
    pub format: Format,
}

impl Bind {
    pub fn new(format: Format) -> Self {
        Bind { format }
    }
}

impl SerializeMessage for Bind {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(BIND_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = vec![];
        let mut portal_name = "".to_string().to_msg_bytes();
        let mut prepared_stmt_name = "".to_string().to_msg_bytes();
        let format = match self.format {
            Format::Text => 0u16,
            Format::Binary => 1u16,
        };
        body.append(&mut portal_name);
        body.append(&mut prepared_stmt_name);
        body.append(&mut 1u16.to_msg_bytes());
        body.append(&mut format.to_msg_bytes());
        body.append(&mut 0u16.to_msg_bytes());
        body.append(&mut 1u16.to_msg_bytes());
        body.append(&mut format.to_msg_bytes());

        body
    }
}

#[derive(Debug)]
pub struct Describe {}

impl Describe {
    fn new() -> Self {
        Describe {}
    }
}

impl Default for Describe {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializeMessage for Describe {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(DESCRIBE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = vec![];
        let description_type = "P";
        let mut portal_name = "".to_string().to_msg_bytes();
        body.append(&mut description_type.as_bytes().to_vec());
        body.append(&mut portal_name);
        body
    }
}

#[derive(Debug)]
pub struct Execute {}

impl Execute {
    fn new() -> Self {
        Execute {}
    }
}

impl Default for Execute {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializeMessage for Execute {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(EXECUTE_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        let mut body = vec![];
        let mut portal_name = "".to_string().to_msg_bytes();
        body.append(&mut portal_name);
        body.append(&mut 0u32.to_msg_bytes());
        body
    }
}

#[derive(Debug)]
pub struct Sync {}

impl Sync {
    fn new() -> Self {
        Sync {}
    }
}

impl Default for Sync {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializeMessage for Sync {
    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        Some(SYNC_MESSAGE_TYPE)
    }

    fn serialize_body(self) -> Vec<u8> {
        vec![]
    }
}
