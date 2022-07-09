#[cfg(test)]
mod tests;
#[cfg(test)]
use crate::messages::DeserializeMessage;
use crate::messages::{SerializeMessage, SerializeMessageBytes};

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

    #[cfg(test)]
    pub fn new_with_version(version: (u16, u16), params: Vec<(String, String)>) -> Self {
        StartupMessage { version, params }
    }
}

impl SerializeMessage for StartupMessage {
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

#[cfg(test)]
impl DeserializeMessage for StartupMessage {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let version_raw: ([u8; 2], [u8; 2]) = (
            body[4..6].try_into().unwrap(),
            body[6..8].try_into().unwrap(),
        );
        let version = (
            u16::from_be_bytes(version_raw.0),
            u16::from_be_bytes(version_raw.1),
        );
        let mut params = Vec::new();
        let clean_iter = body[8..]
            .split(|b| *b == 0)
            .filter(|chunk| !chunk.is_empty());
        let iter_even = clean_iter.to_owned().step_by(2);
        let iter_odd = clean_iter.skip(1).step_by(2);
        let iter = iter_even.zip(iter_odd);
        for (key, value) in iter {
            params.push((
                String::from_utf8(key.to_vec()).unwrap(),
                String::from_utf8(value.to_vec()).unwrap(),
            ));
        }

        StartupMessage::new_with_version(version, params)
    }
}
