use serde::Serialize;
use vhdl_lang::ast::DesignFile;
use vhdl_lang::ast::{AnyDesignUnit, AnyPrimaryUnit, InterfaceDeclaration, ModeIndication};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

use crate::parse_store::get_parsed;
use crate::{decode_typst_arg, decode_typst_arg_id, encode_typst_return};

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[derive(Serialize)]
pub struct PortEntry {
    pub name: String,
    pub mode: String,
    pub port_type: String,
    pub constraint: String,
    pub description: Option<String>,
}

/// go through the design file, find the first entiry and extracts its port list.
/// comments are extracted as description for each port. Both trailing comments
/// (on the same line than the port) and leading comments (on the line before) are
/// detected. If both a trailing and a leading comments are found, only one will
/// be retained. If priority_trailing is true, the trailing comment will be used,
/// and if failse, the leading comment.
pub fn get_port_list_from_design(
    design: DesignFile,
    priority_trailing: bool,
) -> Result<Vec<PortEntry>, String> {
    // Walk the design file looking for entity declarations
    for (tokens, design_unit) in &design.design_units {
        if let AnyDesignUnit::Primary(AnyPrimaryUnit::Entity(entity_decl)) = design_unit {
            // Look for the port clause
            if let Some(port_list) = &entity_decl.port_clause {
                // loop through each port and fill up an array with entries
                let mut entries: Vec<PortEntry> = Vec::new();

                for port in &port_list.items {
                    match port {
                        InterfaceDeclaration::Object(obj_decl) => {
                            // Get mode (direction) and type from the mode indication
                            let (mode_str, type_str, constraint_str) = match &obj_decl.mode {
                                ModeIndication::Simple(simple) => {
                                    let mode = simple
                                        .mode
                                        .as_ref()
                                        .map(|m| m.item.to_string())
                                        .unwrap_or_else(|| "in".to_string()); // default mode is "in"
                                    let typ = simple.subtype_indication.type_mark.to_string();
                                    let constraint = match &simple.subtype_indication.constraint {
                                        Some(constraint) => constraint.to_string(),
                                        None => String::new(),
                                    };
                                    (mode, typ, constraint)
                                }
                                ModeIndication::View(view) => {
                                    let typ = view.name.to_string();
                                    ("view".to_string(), typ, String::new())
                                }
                            };

                            for id in &obj_decl.idents {
                                let name = id.tree.item.name_utf8();
                                let description =
                                    crate::comments::find_object_description(tokens, id.tree.token, priority_trailing, true);
                                entries.push(PortEntry {
                                    name: name,
                                    mode: mode_str.clone(),
                                    port_type: type_str.clone(),
                                    constraint: constraint_str.clone(),
                                    description,
                                });
                            }
                        }
                        InterfaceDeclaration::File(file_decl) => {
                            for id in &file_decl.idents {
                                let name = id.tree.item.name_utf8();
                                let typ = file_decl.subtype_indication.to_string();
                                let description =
                                    crate::comments::find_object_description(tokens, id.tree.token, priority_trailing, true);
                                entries.push(PortEntry {
                                    name: name,
                                    mode: "file".to_owned(),
                                    port_type: typ.clone(),
                                    constraint: String::new(),
                                    description,
                                });
                            }
                        }
                        _ => {}
                    }
                }

                return Ok(entries);
            } else {
                return Ok(Vec::default());
            }
        }
    }

    return Err("no entity found in file".to_owned());
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_port_list(id: &[u8], comment_priority: &[u8]) -> Result<Vec<u8>, String> {
    let id = decode_typst_arg_id(id)?;
    let priority_trailing = decode_typst_arg(comment_priority)? == "trailing";

    let designfile = get_parsed(id)?;

    let portlist = get_port_list_from_design(designfile, priority_trailing)?;

    encode_typst_return(&portlist)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use vhdl_lang::{Source, VHDLParser, VHDLStandard};

    fn parse_test_file() -> DesignFile {
        let contents = std::fs::read_to_string("test/test.vhd").unwrap();
        let parser = VHDLParser::new(VHDLStandard::VHDL2008);
        let mut diagnostics = Vec::new();
        parser.parse_design_source(
            &Source::inline(Path::new("test.vhd"), &contents),
            &mut diagnostics,
        )
    }

    #[test]
    fn test_port_descriptions() {
        let design = parse_test_file();
        let ports = get_port_list_from_design(design, true).unwrap();

        // Ports with same-line comments
        assert_eq!(ports[0].name, "clock");
        assert_eq!(ports[0].description, Some("main clock".to_owned()));

        assert_eq!(ports[1].name, "sreset");
        assert_eq!(ports[1].description, Some("main reset, synchronous, active high".to_owned()));

        assert_eq!(ports[2].name, "output_a");
        assert_eq!(ports[2].description, Some("a regular output".to_owned()));

        // Port with a solo comment on the line before
        assert_eq!(ports[3].name, "input_b");
        assert_eq!(ports[3].description, Some("one input".to_owned()));

        // Ports without descriptions
        assert_eq!(ports[4].name, "output_c");
        assert_eq!(ports[4].description, Some("another output".to_owned()));

        assert_eq!(ports[5].name, "input_d");
        assert_eq!(ports[5].description, Some("another input".to_owned()));

        assert_eq!(ports[6].name, "data_in");
        assert_eq!(ports[6].description, None);

        assert_eq!(ports[7].name, "data_out");
        assert_eq!(ports[7].description, Some("data out".to_owned()));
    }
}
