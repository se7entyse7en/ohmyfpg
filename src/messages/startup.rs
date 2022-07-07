#[cfg(test)]
mod tests;

use crate::messages::{Message, MessageBytesSerialize};

static PROTOCOL_MAJOR_VALUE: u16 = 3;
static PROTOCOL_MINOR_VALUE: u16 = 0;

#[derive(Debug)]
pub struct StartupMessage {
    pub version: (u16, u16),
    pub params: Vec<(String, String)>,
}

impl StartupMessage {
    pub fn new(params: Vec<(String, String)>) -> Self {
        StartupMessage {
            version: (PROTOCOL_MAJOR_VALUE, PROTOCOL_MINOR_VALUE),
            params,
        }
    }
}

impl Message for StartupMessage {
    fn serialize_body(self) -> Vec<u8> {
        let mut body = [self.version.0.to_msg_bytes(), self.version.1.to_msg_bytes()]
            .concat()
            .to_vec();

        let mut params = Vec::new();
        for param in self.params.into_iter() {
            params.append(&mut param.0.to_msg_bytes());
            params.append(&mut param.1.to_msg_bytes());
        }
        params.push(0x00);
        body.append(&mut params);
        body
    }
}
