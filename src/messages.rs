pub mod authentication;
pub mod startup;
use authentication::AuthenticationSASL;

pub trait SerializeMessage: Sized {
    fn serialize(self) -> Vec<u8> {
        let mut ser = self.serialize_msg_type().unwrap_or_default();
        let mut body = self.serialize_body();
        let count: u32 = (4 + body.len()).try_into().unwrap();
        ser.append(&mut count.to_msg_bytes());
        ser.append(&mut body);

        ser
    }

    fn serialize_msg_type(&self) -> Option<Vec<u8>> {
        self.get_msg_type().map(|s| s.to_owned().to_vec())
    }

    fn get_msg_type(&self) -> Option<&[u8; 1]> {
        None
    }

    fn serialize_body(self) -> Vec<u8>;
}

pub trait DeserializeMessage {
    fn deserialize_body(body: Vec<u8>) -> Self;
}

pub trait SerializeMessageBytes {
    fn to_msg_bytes(self) -> Vec<u8>;
}

impl SerializeMessageBytes for String {
    fn to_msg_bytes(self) -> Vec<u8> {
        let mut ser = self.into_bytes();
        ser.push(0x00);
        ser
    }
}

impl SerializeMessageBytes for u16 {
    fn to_msg_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl SerializeMessageBytes for u32 {
    fn to_msg_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

#[derive(Debug)]
pub enum BackendMessage {
    AuthenticationSASL(AuthenticationSASL),
}
