use serde::Serialize;
use vhdl_lang::{
    HasTokenSpan, Token,
    ast::{
        AnyDesignUnit, AnyPrimaryUnit, AnySecondaryUnit,
        Declaration::{self, Object, Type},
        DesignFile,
        ObjectClass::{Constant, Signal},
        TypeDefinition::{Array, Enumeration, Record, Subtype},
        token_range::WithTokenSpan,
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

/// Describes a type
#[derive(Serialize)]
pub struct TypeDescription {
    /// name of the new type
    pub name: String,
    /// enumration, array, record....
    pub kind: String,
    /// type definition
    pub definition: TypeDefinition,
    /// type description (comment)
    pub description: Option<String>,
}

/// Describes a type definition
#[derive(Serialize)]
pub enum TypeDefinition {
    Enumeration(Vec<TypeDefinitionEnumElement>),
    Array(TypeDefinitionArray),
    Record(Vec<TypeDefinitionRecordElement>),
    SubType(TypeDefinitionSubtype),
}

/// Describes an enumeration element
#[derive(Serialize)]
pub struct TypeDefinitionEnumElement {
    /// name of the enum element
    pub element_name: String,
    /// description (comment)
    pub description: Option<String>,
}

/// Describes an array type definition
#[derive(Serialize)]
pub struct TypeDefinitionArray {
    /// array range
    pub range: String,
    /// array element type
    pub subtype: String,
}

/// Describes a record element
#[derive(Serialize)]
pub struct TypeDefinitionRecordElement {
    /// name of the record element
    pub element_name: String,
    /// type of the record element
    pub element_type: String,
    /// description (comment)
    pub description: Option<String>,
}

/// Describes a subtype
#[derive(Serialize)]
pub struct TypeDefinitionSubtype {
    /// type
    pub subtype: String,
    /// constraint (x downto y)
    pub constraint: Option<String>,
}

/// Return from the analysys function
#[derive(Serialize)]
pub struct Declarations {
    /// signals list
    pub signals: Vec<ObjectDescription>,
    /// constants list
    pub constants: Vec<ObjectDescription>,
    /// types list
    pub types: Vec<TypeDescription>,
}
/// return the list of declarations in a design file
pub fn get_declarations(
    design: DesignFile,
    priority_trailing: bool,
) -> Result<Declarations, String> {
    // loop through architectures or packages
    for (tokens, design_unit) in &design.design_units {
        match design_unit {
            AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(architecture)) => {
                return explore_declarations(&architecture.decl, tokens, priority_trailing);
            }
            AnyDesignUnit::Primary(AnyPrimaryUnit::Package(package)) => {
                return explore_declarations(&package.decl, tokens, priority_trailing);
            }
            _ => {}
        }
    }

    Err("no architecture or package found".to_owned())
}

/// go through a list of declarations
fn explore_declarations(
    declarations: &Vec<WithTokenSpan<Declaration>>,
    tokens: &Vec<Token>,
    priority_trailing: bool,
) -> Result<Declarations, String> {
    let mut signals_list: Vec<ObjectDescription> = Vec::default();
    let mut constants_list: Vec<ObjectDescription> = Vec::default();
    let mut types_list: Vec<TypeDescription> = Vec::default();

    for declaration in declarations {
        match &declaration.item {
            Object(object_declaration) => {
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
            Type(type_declaration) => {
                let type_name = type_declaration.ident.to_string();

                // depending on the kind of type declaration, make the relevant description structure
                let extracted_definition = match &type_declaration.def {
                    Enumeration(enumerations) => Some((
                        "enumeration",
                        TypeDefinition::Enumeration(
                            enumerations
                                .iter()
                                .map(|e| TypeDefinitionEnumElement {
                                    element_name: e.to_string(),
                                    description: find_object_description(
                                        tokens,
                                        e.get_start_token(),
                                        priority_trailing,
                                        false,
                                    ),
                                })
                                .collect(),
                        ),
                    )),
                    Array(indices, _, subtype) => {
                        let indices_str: Vec<String> =
                            indices.iter().map(|index| index.to_string()).collect();

                        Some((
                            "array",
                            TypeDefinition::Array(TypeDefinitionArray {
                                range: indices_str.join(","),
                                subtype: subtype.to_string(),
                            }),
                        ))
                    }
                    Record(elements) => {
                        // each element declaration can be in fact several elements, because you can have more than one with the same type
                        // so we need two level of iterators
                        Some((
                            "record",
                            TypeDefinition::Record(
                                elements
                                    .iter()
                                    .map(|e| {
                                        let element_type = e.subtype.to_string();
                                        let element_description = find_object_description(
                                            tokens,
                                            e.get_start_token(),
                                            priority_trailing,
                                            false,
                                        );

                                        e.idents.iter().map(move |f| TypeDefinitionRecordElement {
                                            element_name: f.to_string(),
                                            element_type: element_type.clone(),
                                            description: element_description.clone(),
                                        })
                                    })
                                    .flatten()
                                    .collect(),
                            ),
                        ))
                    }
                    Subtype(subtype) => Some((
                        "subtype",
                        TypeDefinition::SubType(TypeDefinitionSubtype {
                            subtype: subtype.type_mark.to_string(),
                            constraint: subtype
                                .constraint
                                .as_ref()
                                .map(|constraint| constraint.to_string()),
                        }),
                    )),
                    _ => None,
                };

                // add the extracted type to the list
                if let Some((type_kind, type_definition)) = extracted_definition {
                    types_list.push(TypeDescription {
                        name: type_name,
                        kind: type_kind.to_owned(),
                        definition: type_definition,
                        description: find_object_description(
                            tokens,
                            type_declaration.get_start_token(),
                            priority_trailing,
                            false,
                        ),
                    })
                }
            }
            _ => {}
        }
    }

    Ok(Declarations {
        signals: signals_list,
        constants: constants_list,
        types: types_list,
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

    fn parse_test_package() -> DesignFile {
        let contents = std::fs::read_to_string("test/test_pkg.vhd").unwrap();
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
        let sigs_consts = get_declarations(design, false).unwrap();

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

    #[test]
    fn test_types() {
        let design = parse_test_file();
        let types = get_declarations(design, false).unwrap().types;

        // check the types list
        assert_eq!(types.len(), 4);

        // check the fsm type
        let fsm_type = types.get(0).unwrap();
        assert_eq!(fsm_type.name, "fsm_t");
        assert_eq!(fsm_type.kind, "enumeration");
        assert_eq!(
            fsm_type.description,
            Some("enumeration type for the FSM".to_owned())
        );

        if let TypeDefinition::Enumeration(enumeration) = &fsm_type.definition {
            assert_eq!(enumeration.len(), 4);
            assert_eq!(enumeration.get(0).unwrap().element_name, "reset");
            assert_eq!(
                enumeration.get(0).unwrap().description,
                Some("reset, initial state".to_owned())
            );
            assert_eq!(enumeration.get(1).unwrap().element_name, "idle");
            assert_eq!(
                enumeration.get(1).unwrap().description,
                Some("state when not doing anything".to_owned())
            );
            assert_eq!(enumeration.get(2).unwrap().element_name, "read_input");
            assert_eq!(
                enumeration.get(2).unwrap().description,
                Some("check what's on the inputs".to_owned())
            );
            assert_eq!(enumeration.get(3).unwrap().element_name, "write_output");
            assert_eq!(
                enumeration.get(3).unwrap().description,
                Some("do something on the outputs".to_owned())
            );
        } else {
            panic!("wrong type definition")
        }

        // check the fsm wrapper type
        let fsm_wrapper_type = types.get(1).unwrap();
        assert_eq!(fsm_wrapper_type.name, "fsm_wrapper_t");
        assert_eq!(fsm_wrapper_type.kind, "record");
        assert_eq!(
            fsm_wrapper_type.description,
            Some(
                "a wrapper around the fsm type to test record element access in FSM detection"
                    .to_owned()
            )
        );

        if let TypeDefinition::Record(record) = &fsm_wrapper_type.definition {
            assert_eq!(record.len(), 1);
            assert_eq!(record.get(0).unwrap().element_name, "state");
            assert_eq!(record.get(0).unwrap().element_type, "fsm_t");
            assert_eq!(
                record.get(0).unwrap().description,
                Some("the actual fsm type".to_owned())
            );
        } else {
            panic!("wrong type definition")
        }

        // check the subtype
        let byte_type = types.get(2).unwrap();
        assert_eq!(byte_type.name, "byte_t");
        assert_eq!(byte_type.kind, "subtype");
        assert_eq!(byte_type.description, Some("a subtype".to_owned()));

        if let TypeDefinition::SubType(subtype) = &byte_type.definition {
            assert_eq!(subtype.subtype, "std_logic_vector");
            assert_eq!(subtype.constraint, Some("(7 downto 0)".to_owned()));
        } else {
            panic!("wrong type definition")
        }

        // check the array type
        let matrix_type = types.get(3).unwrap();
        assert_eq!(matrix_type.name, "matrix_t");
        assert_eq!(matrix_type.kind, "array");
        assert_eq!(matrix_type.description, Some("an array".to_owned()));

        if let TypeDefinition::Array(array) = &matrix_type.definition {
            assert_eq!(array.range, "31 downto 0,7 downto 0");
            assert_eq!(array.subtype, "byte_t");
        } else {
            panic!("wrong type definition")
        }
    }

    #[test]
    fn test_types_in_package() {
        let design = parse_test_package();
        let types = get_declarations(design, false).unwrap().types;

        // check the types list
        assert_eq!(types.len(), 1);

        // check the fsm type
        let status_type = types.get(0).unwrap();
        assert_eq!(status_type.name, "test_status_t");
        assert_eq!(status_type.kind, "enumeration");
        assert_eq!(
            status_type.description,
            Some("an enumeration type without comments on each element".to_owned())
        );

        if let TypeDefinition::Enumeration(enumeration) = &status_type.definition {
            assert_eq!(enumeration.len(), 2);
            assert_eq!(enumeration.get(0).unwrap().element_name, "working");
            assert_eq!(enumeration.get(0).unwrap().description, None);
            assert_eq!(enumeration.get(1).unwrap().element_name, "not_working");
            assert_eq!(enumeration.get(1).unwrap().description, None);
        } else {
            panic!("wrong type definition")
        }
    }
}

/// typst plugin function to find the architectures in a file and return a structure with its constants and signals
#[allow(dead_code)]
#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_declarations_struct(
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

    let sigs_consts = get_declarations(designfile, priority_trailing)?;

    encode_typst_return(&sigs_consts)
}
