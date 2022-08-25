#[derive(Debug)]
pub struct PgType {
    pub oid: u32,
    pub name: String,
    pub size: Option<u8>,
}

impl PgType {
    pub fn new(oid: u32, name: String, size: Option<u8>) -> Self {
        PgType { oid, name, size }
    }
}
