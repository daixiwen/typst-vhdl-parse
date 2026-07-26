pub mod comments;
pub mod declarations;
pub mod entity_declaration;
pub mod fsm;
pub mod instances;
pub mod parse_store;

use serde::{Serialize, de::DeserializeOwned};

/// decode a typst argument from the bytes type to string
pub fn decode_typst_arg(arg: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(arg).map_err(|e| e.to_string())
}

/// decode a typst argument from the bytes type to a u64 ID
pub fn decode_typst_arg_id(arg: &[u8]) -> Result<u64, String> {
    let str_arg = std::str::from_utf8(arg).map_err(|e| e.to_string())?;

    u64::from_str_radix(str_arg, 10).map_err(|e| e.to_string())
}

/// decode a typst argument from the bytes type to a struct
pub fn decode_typst_arg_struct<T: DeserializeOwned>(arg: &[u8]) -> Result<T, String> {
    let read_struct: T = ciborium::from_reader(arg).map_err(|e| e.to_string())?;

    Ok(read_struct)
}

/// encode a reply to typst from any type
pub fn encode_typst_return<T: Serialize>(arg: &T) -> Result<Vec<u8>, String> {
    let mut buffer: Vec<u8> = Vec::new();
    ciborium::into_writer(&arg, &mut buffer).map_err(|e| e.to_string())?;

    Ok(buffer)
}
