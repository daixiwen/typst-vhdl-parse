use serde::Serialize;
use vhdl_lang::ast::DesignFile;
use vhdl_lang::ast::{AnyDesignUnit, AnyPrimaryUnit, InterfaceDeclaration, ModeIndication};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

use crate::parse_store::{get_content, get_parsed};
use crate::{decode_typst_arg_id, encode_typst_return};

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[derive(Serialize)]
pub struct PortEntry {
    pub name: String,
    pub mode: String,
    pub port_type: String,
    pub constraint: String,
    pub description: String,
}

pub fn get_port_list_from_design(design: DesignFile, content: &str) -> Result<Vec<PortEntry>, String> {
    // Walk the design file looking for entity declarations
    for (_tokens, design_unit) in &design.design_units {
        let entity_decl = match design_unit {
            AnyDesignUnit::Primary(AnyPrimaryUnit::Entity(entity)) => entity,
            _ => continue,
        };

        // Look for the port clause
        if let Some(port_list) = &entity_decl.port_clause {
            // loop through each port and fill up an array with entries
            let mut entries: Vec<PortEntry> = Vec::new();

            for port in &port_list.items {
                match port {
                    InterfaceDeclaration::Object(obj_decl) => {
                        // Get port name(s) - a single declaration can declare
                        // multiple ports of the same type
                        let names: Vec<String> = obj_decl
                            .idents
                            .iter()
                            .map(|id| id.tree.item.name_utf8())
                            .collect();

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

                        for name in names {
                            let description = find_port_description(content, &name);
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
                        let names: Vec<String> = file_decl
                            .idents
                            .iter()
                            .map(|id| id.tree.item.name_utf8())
                            .collect();
                        let typ = file_decl.subtype_indication.to_string();
                        for name in names {
                            let description = find_port_description(content, &name);
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

    return Err("no entity found in file".to_owned());
}

/// Find the description (comment) associated with a port declaration in the VHDL source.
/// A description is either a comment on the same line as the port (`-- comment`),
/// or a solo comment on the line immediately before the port declaration.
/// If no description is found, returns an empty string.
fn find_port_description(content: &str, port_name: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // Split the line at the first '--' to separate code from comment
        let (code_part, same_line_comment) = match line.find("--") {
            Some(pos) => (&line[..pos], Some(line[pos + 2..].trim())),
            None => (*line, None),
        };

        // Check if this line contains the port declaration.
        // A port declaration line must contain a colon (for the mode indication)
        // and the port name must appear as a word before the colon.
        if let Some(colon_pos) = code_part.find(':') {
            let before_colon = &code_part[..colon_pos];
            if contains_word(before_colon, port_name) {
                // Same-line comment takes priority
                if let Some(comment) = same_line_comment {
                    if !comment.is_empty() {
                        return comment.to_string();
                    }
                }

                // Check previous line for a solo comment
                if i > 0 {
                    let prev_line = lines[i - 1].trim();
                    if prev_line.starts_with("--") {
                        return prev_line[2..].trim().to_string();
                    }
                }

                return String::new();
            }
        }
    }

    String::new()
}

/// Check if `word` appears as a whole word in `text`.
/// Words are separated by characters that are not alphanumeric or underscore.
fn contains_word(text: &str, word: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|w| w == word)
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_port_list(id: &[u8]) -> Result<Vec<u8>, String> {
    let id = decode_typst_arg_id(id)?;
    let designfile = get_parsed(id)?;
    let content = get_content(id)?;

    let portlist = get_port_list_from_design(designfile, &content)?;

    encode_typst_return(&portlist)
}
