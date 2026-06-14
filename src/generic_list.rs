use serde::Serialize;
use vhdl_lang::ast::DesignFile;
use vhdl_lang::ast::{AnyDesignUnit, AnyPrimaryUnit, InterfaceDeclaration, ModeIndication};

use crate::comments::find_object_description;
use crate::parse_store::get_parsed;
use crate::{decode_typst_arg, decode_typst_arg_id, encode_typst_return};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

// Describes a single generic
#[derive(Serialize)]
pub struct GenericEntry {
    /// generic name
    pub name: String,
    /// type           
    pub generic_type: String,
    /// constraint (x downto y)      
    pub constraint: Option<String>,
    /// comment or description
    pub description: Option<String>,
    /// default value
    pub default_value: Option<String>,
}

/// go through the design file, find the first entiry and extracts its generics list.
/// comments are extracted as description for each generic. Both trailing comments
/// (on the same line than the generic) and leading comments (on the line before) are
/// detected. If both a trailing and a leading comments are found, only one will
/// be retained. If priority_trailing is true, the trailing comment will be used,
/// and if failse, the leading comment.
pub fn get_generic_list_from_design(
    design: DesignFile,
    priority_trailing: bool,
) -> Result<Vec<GenericEntry>, String> {
    // Walk the design file looking for entity declarations
    for (tokens, design_unit) in &design.design_units {
        if let AnyDesignUnit::Primary(AnyPrimaryUnit::Entity(entity_decl)) = design_unit {
            // Look for the generics clause
            if let Some(generic_list) = &entity_decl.generic_clause {
                // loop through each generic and fill up an array with entries
                let mut entries: Vec<GenericEntry> = Vec::new();

                for generic in &generic_list.items {
                    // for now I'm only decoding generics with ModeInfication::Simple, I don't know if some exotic
                    // code could produce something else
                    if let InterfaceDeclaration::Object(obj_decl) = generic {
                        if let ModeIndication::Simple(simple) = &obj_decl.mode {
                            // extract all the information to create the GenericEntry
                            let type_str = simple.subtype_indication.type_mark.to_string();
                            let constraint = simple
                                .subtype_indication
                                .constraint
                                .as_ref()
                                .map(|constraint| constraint.to_string());
                            let default = simple
                                .expression
                                .as_ref()
                                .map(|expression| expression.to_string());

                            for id in &obj_decl.idents {
                                let name = id.tree.item.name_utf8();
                                let description = find_object_description(
                                    tokens,
                                    id.tree.token,
                                    priority_trailing,
                                    true,
                                );

                                entries.push(GenericEntry {
                                    name: name,
                                    generic_type: type_str.clone(),
                                    constraint: constraint.clone(),
                                    description: description.clone(),
                                    default_value: default.clone(),
                                });
                            }
                        }
                    }
                }

                return Ok(entries);
            }
            return Ok(Vec::default());
        }
    }

    return Err("no entity found in file".to_owned());
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
    fn test_generic_descriptions() {
        let design = parse_test_file();
        let generics = get_generic_list_from_design(design, true).unwrap();

        assert_eq!(generics.len(), 2);
        assert_eq!(generics[0].name, "flag");
        assert_eq!(generics[0].generic_type, "boolean");
        assert_eq!(generics[0].constraint, None);
        assert_eq!(
            generics[0].description,
            Some("a flag, can be true or false".to_owned())
        );
        assert_eq!(generics[0].default_value, None);
        assert_eq!(generics[1].name, "value");
        assert_eq!(generics[1].generic_type, "std_logic_vector");
        assert_eq!(generics[1].constraint, Some("(15 downto 0)".to_owned()));
        assert_eq!(
            generics[1].description,
            Some("a value with a default".to_owned())
        );
        assert_eq!(generics[1].default_value, Some("x\"DEAD\"".to_owned()));
    }
}

/// typst plugin function to extract the generics from the file and return them as
/// an array of GenericEntry
#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_generic_list(
    id: &[u8],
    file_name: &[u8],
    vhdl_standard: &[u8],
    contents: &[u8],
    comment_priority: &[u8],
) -> Result<Vec<u8>, String> {
    let id = decode_typst_arg_id(id)?;
    let file_name = decode_typst_arg(file_name)?;
    let vhdl_standard = decode_typst_arg(vhdl_standard)?;
    let contents = decode_typst_arg(contents)?;
    let priority_trailing = decode_typst_arg(comment_priority)? == "trailing";

    let designfile = get_parsed(id, file_name, vhdl_standard, contents)?;

    let genericlist = get_generic_list_from_design(designfile, priority_trailing)?;

    encode_typst_return(&genericlist)
}
