use crate::messages::DeserializeMessage;

pub const ROW_DESCRIPTION_MESSAGE_TYPE: &[u8; 1] = b"T";
pub const DATA_ROW_MESSAGE_TYPE: &[u8; 1] = b"D";
pub const COMMAND_COMPLETE_MESSAGE_TYPE: &[u8; 1] = b"C";

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

#[derive(Debug)]
pub struct DataRow {
    pub columns: Vec<Option<Vec<u8>>>,
}

impl DataRow {
    pub fn new(columns: Vec<Option<Vec<u8>>>) -> Self {
        DataRow { columns }
    }
}

impl DeserializeMessage for DataRow {
    fn deserialize_body(body: Vec<u8>) -> Self {
        let cols_first_index = 2;
        let raw_cols_count: [u8; 2] = body[0..cols_first_index].try_into().unwrap();
        let cols_count = u16::from_be_bytes(raw_cols_count);
        let cols_values = if cols_count == 0 {
            vec![]
        } else {
            let mut col_idx_start = cols_first_index;
            let mut cols_values: Vec<Option<Vec<u8>>> = Vec::with_capacity(cols_count.into());
            for _ in 1..cols_count + 1 {
                let col_idx_end = col_idx_start + 4;
                let raw_value_len: [u8; 4] = body[col_idx_start..col_idx_end].try_into().unwrap();
                let value_len = i32::from_be_bytes(raw_value_len);

                if value_len < 0 {
                    cols_values.push(None);
                    col_idx_start = col_idx_end;
                } else {
                    let value_idx_start: usize = col_idx_end;
                    let value_idx_end: usize = value_idx_start + value_len as usize;
                    cols_values.push(Some(body[value_idx_start..value_idx_end].to_vec()));
                    col_idx_start = col_idx_end + value_len as usize;
                }
            }
            cols_values
        };

        DataRow::new(cols_values)
    }
}

#[derive(Debug)]
pub struct CommandComplete {
    pub tag: String,
}

impl CommandComplete {
    pub fn new(tag: String) -> Self {
        CommandComplete { tag }
    }
}

impl DeserializeMessage for CommandComplete {
    fn deserialize_body(body: Vec<u8>) -> Self {
        CommandComplete::new(String::from_utf8(body[..body.len() - 1].to_vec()).unwrap())
    }
}
