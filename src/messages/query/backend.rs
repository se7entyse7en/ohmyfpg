use crate::messages::DeserializeMessage;

pub const ROW_DESCRIPTION_MESSAGE_TYPE: &[u8; 1] = b"T";

#[derive(Debug)]
pub struct FieldDescription {
    pub name: String,
    pub data_type_oid: u32,
}

impl FieldDescription {
    pub fn new(name: String, data_type_oid: u32) -> Self {
        FieldDescription {
            name,
            data_type_oid,
        }
    }
}

#[derive(Debug)]
pub struct RowDescription {
    pub fields: Vec<FieldDescription>,
}

impl RowDescription {
    pub fn new(fields: Vec<FieldDescription>) -> Self {
        RowDescription { fields }
    }
}

impl DeserializeMessage for RowDescription {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let fields_first_index = 2;
        let raw_fields_count: [u8; 2] = body[0..fields_first_index].try_into().unwrap();
        let fields_count = u16::from_be_bytes(raw_fields_count);
        let mut name_idx_start = fields_first_index;
        let mut fields_desc: Vec<FieldDescription> = Vec::with_capacity(fields_count.into());
        for _ in 1..fields_count + 1 {
            let name_idx_shift = body[name_idx_start..].iter().position(|&b| b == 0).unwrap();
            let name_idx_end = name_idx_start + name_idx_shift;
            let name = String::from_utf8(body[name_idx_start..name_idx_end].to_vec()).unwrap();
            let data_type_oid_idx_start = name_idx_end + 7;

            let data_type_oid_idx_end = data_type_oid_idx_start + 4;
            let raw_data_type_oid: [u8; 4] = body[data_type_oid_idx_start..data_type_oid_idx_end]
                .try_into()
                .unwrap();
            let data_type_oid = u32::from_be_bytes(raw_data_type_oid);

            name_idx_start = name_idx_end + 19;
            fields_desc.push(FieldDescription::new(name, data_type_oid));
        }
        RowDescription::new(fields_desc)
    }
}
