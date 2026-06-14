// this file handles the parsing of VHDL file and the local storage of the parsed structures

use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use vhdl_lang::ast::DesignFile;
use vhdl_lang::{Source, VHDLParser, VHDLStandard};

use crate::{decode_typst_arg, encode_typst_return};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

// context store: storage of the parsed VHDL data
lazy_static! {
    // We associate each parsed data to a unique ID, which is made by hashing the original
    // contents. The ID is what is returned to typst, and used as first arguments in all
    // the other function calls to access the parsed data. If the VHDL contents change during
    // live editing, the hash also changes and typst will not use the cached results of the
    // different functions in this package.
    static ref PARSED_STORE: Mutex<HashMap<u64, DesignFile>> = Mutex::new(HashMap::new());

    // we also need to store a file name to id mapping, so that during live editing if a new
    // version of the file is parsed, we can remove the old version and prevent a leak
    static ref ID_STORE: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());
}

/// parse a VHDL file and stores it in cache. Returns its ID and an array of diagnostic messages
pub fn parse_content(
    file_name: &str,
    vhdl_standard: &str,
    contents: &str,
) -> Result<(u64, Vec<String>), String> {
    let parser = VHDLParser::new(match vhdl_standard {
        "1993" | "93" => VHDLStandard::VHDL1993,
        "2019" | "19" => VHDLStandard::VHDL2019,
        _ => VHDLStandard::VHDL2008,
    });

    // Parse the VHDL file
    let mut diagnostics = Vec::new();
    let result = parser.parse_design_source(
        &Source::inline(std::path::Path::new(file_name), contents),
        &mut diagnostics,
    );

    // calculate the id from the VHDL contents
    let mut hash = std::hash::DefaultHasher::new();
    contents.hash(&mut hash);
    let id = hash.finish();

    // get access to both indexes
    let mut id_index = ID_STORE.lock().unwrap();
    let mut parsed_index = PARSED_STORE.lock().unwrap();

    // check if we have parsed a file with the same name before
    if let Some(old_id) = id_index.get(file_name) {
        if parsed_index.remove(old_id).is_none() {
            return Err("Couldn't find the old parsed data".to_owned());
        }
    }

    // store the parsed data
    parsed_index.insert(id, result);

    // update the ID index
    id_index.insert(file_name.to_owned(), id);

    // turn the diagnostics into a vector of strings
    let messages: Vec<String> = diagnostics
        .iter()
        .map(|x| format!("{}: {}", x.pos.range.start.line, x.message))
        .collect();

    return Ok((id, messages));
}

/// looks for a previously parsed file in the storage. If not found, parse it again
pub fn get_parsed(
    id: u64,
    file_name: &str,
    vhdl_standard: &str,
    contents: &str,
) -> Result<DesignFile, String> {
    let parsed_index = PARSED_STORE.lock().unwrap();

    match parsed_index.get(&id) {
        Some(design) => Ok(design.clone()),

        None => {
            // parse the file again and verify we still get the same id
            let (new_id, _) = parse_content(file_name, vhdl_standard, contents)?;
            if new_id != id {
                Err("file contents different from cached version".to_owned())
            } else {
                match parsed_index.get(&id) {
                    Some(design) => Ok(design.clone()),
                    None => Err("unknown cache error".to_owned()), // this should never happen
                }
            }
        }
    }
}

#[derive(Serialize)]
struct ParseResponse {
    id: String,
    messages: Vec<String>,
}

/// typst plugin function to parse a VHDL file
#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn parse(file_name: &[u8], vhdl_standard: &[u8], contents: &[u8]) -> Result<Vec<u8>, String> {
    let file_name = decode_typst_arg(file_name)?;
    let vhdl_standard = decode_typst_arg(vhdl_standard)?;
    let contents = decode_typst_arg(contents)?;

    let (id, messages) = parse_content(file_name, vhdl_standard, contents)?;

    let response = ParseResponse {
        id: id.to_string(),
        messages: messages,
    };

    encode_typst_return(&response)
}
