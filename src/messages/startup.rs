#[cfg(test)]
mod tests;

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

    pub fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let mut version_bytes = [self.version.0.to_be_bytes(), self.version.1.to_be_bytes()]
            .concat()
            .to_vec();

        let mut params_bytes = Vec::new();
        for param in self.params.into_iter() {
            params_bytes.append(&mut param.0.into_bytes());
            params_bytes.push(0x00);
            params_bytes.append(&mut param.1.into_bytes());
            params_bytes.push(0x00);
        }
        params_bytes.push(0x00);

        let length: u32 = (4 + version_bytes.len() + params_bytes.len())
            .try_into()
            .unwrap();

        bytes.append(&mut length.to_be_bytes().to_vec());
        bytes.append(&mut version_bytes);
        bytes.append(&mut params_bytes);

        bytes
    }
}
