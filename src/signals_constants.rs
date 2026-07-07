use serde::Serialize;
use vhdl_lang::{
    HasTokenSpan,
    ast::{
        AnyDesignUnit, AnySecondaryUnit,
        Declaration::Object,
        DesignFile,
        ObjectClass::{Constant, Signal},
    },
};

use crate::comments::find_object_description;
use crate::parse_store::get_parsed;
use crate::{decode_typst_arg, decode_typst_arg_id, encode_typst_return};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

/// Describes a signal or a constant
#[derive(Serialize)]
pub struct ObjectDescription {
    /// signal or constant name
    pub name: String,
    /// signal or constant type
    pub object_type: String,
    /// constraint (x downto y)
    pub constraint: Option<String>,
    /// constant value or signal initial value
    pub expression: Option<String>,
    /// a description for the signal (comment)
    pub description: Option<String>,
}

/// Return from the analysys function
#[derive(Serialize)]
pub struct SignalsConstants {
    /// signals list
    pub signals: Vec<ObjectDescription>,
    /// constants list
    pub constants: Vec<ObjectDescription>,
}
/// return the list of signals in a design file
pub fn get_signals_constants(
    design: DesignFile,
    priority_trailing: bool,
) -> Result<SignalsConstants, String> {
    let mut signals_list: Vec<ObjectDescription> = Vec::default();
    let mut constants_list: Vec<ObjectDescription> = Vec::default();

    // loop through architectures
    for (tokens, design_unit) in &design.design_units {
        if let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(architecture)) = design_unit
        {
            // loop through declarations in the architecture
            for declaration in &architecture.decl {
                if let Object(object_declaration) = &declaration.item {
                    // extract details about the objects
                    let object_type = object_declaration.subtype_indication.type_mark.to_string();
                    let constraint = object_declaration
                        .subtype_indication
                        .constraint
                        .as_ref()
                        .map(|constraint| constraint.to_string());
                    let expression = object_declaration
                        .expression
                        .as_ref()
                        .map(|e| e.to_string());
                    let description = find_object_description(
                        tokens,
                        declaration.get_start_token(),
                        priority_trailing,
                        false,
                    );

                    // creates one ObjectDescription for each object
                    let mut objects_list: Vec<ObjectDescription> = object_declaration
                        .idents
                        .iter()
                        .map(|ident| ObjectDescription {
                            name: ident.tree.item.to_string(),
                            object_type: object_type.clone(),
                            constraint: constraint.clone(),
                            expression: expression.clone(),
                            description: description.clone(),
                        })
                        .collect();

                    // add the list to the correct list, depending on whether it's a signal or a constant
                    match object_declaration.class {
                        Signal => {
                            signals_list.append(&mut objects_list);
                        }
                        Constant => {
                            constants_list.append(&mut objects_list);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(SignalsConstants {
        signals: signals_list,
        constants: constants_list,
    })
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
    fn test_sigs_consts() {
        let design = parse_test_file();
        let sigs_consts = get_signals_constants(design, false).unwrap();

        // check the constants list
        assert_eq!(sigs_consts.constants.len(), 1);

        // check the only constant
        let my_constant = sigs_consts.constants.get(0).unwrap();
        assert_eq!(my_constant.name, "c_ones");
        assert_eq!(my_constant.object_type, "std_logic_vector");
        assert_eq!(my_constant.constraint, Some("(31 downto 0)".to_owned()));
        assert_eq!(my_constant.expression, Some("(others => '1')".to_owned()));
        assert_eq!(my_constant.description, Some("a constant".to_owned()));

        // check the signals list
        assert_eq!(sigs_consts.signals.len(), 2);

        // check the first signal
        let my_signal = sigs_consts.signals.get(0).unwrap();
        assert_eq!(my_signal.name, "fsm");
        assert_eq!(my_signal.object_type, "fsm_wrapper_t");
        assert_eq!(my_signal.constraint, None);
        assert_eq!(my_signal.expression, None);
        assert_eq!(
            my_signal.description,
            Some("state machine signal".to_owned())
        );

        // check the second signal
        let my_signal = sigs_consts.signals.get(1).unwrap();
        assert_eq!(my_signal.name, "mysignal");
        assert_eq!(my_signal.object_type, "unsigned");
        assert_eq!(my_signal.constraint, Some("(15 downto 0)".to_owned()));
        assert_eq!(my_signal.expression, Some("(others => '0')".to_owned()));
        assert_eq!(
            my_signal.description,
            Some("a signal with a comment on the same line".to_owned())
        );
    }
}

/// typst plugin function to find the architectures in a file and return a structure with its constants and signals
#[allow(dead_code)]
#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_signals_constants_struct(
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

    let sigs_consts = get_signals_constants(designfile, priority_trailing)?;

    encode_typst_return(&sigs_consts)
}
