use vhdl_lang::{HasTokenSpan, ast::{AnyDesignUnit, AnySecondaryUnit, Declaration::Object, DesignFile, ObjectClass::{Signal,Constant}}};

use crate::comments::find_object_description;


/// Describes a signal or a constant
pub struct ObjectDescription {
    /// signal or constant name
    pub name: String,
    /// signal or constant type
    pub object_type: String,
    /// constant value or signal initial value
    pub expression: Option<String>,
    /// a description for the signal (comment)
    pub description: Option<String>,
}

/// Return from the analysys function
pub struct SignalsConstants {
    /// signals list
    pub signals: Vec<ObjectDescription>,
    /// constants list
    pub constants: Vec<ObjectDescription>,
}
/// return the list of signals in a design file
pub fn get_signals_constants(design: DesignFile, priority_trailing: bool) -> Result<SignalsConstants, String> {

    let mut signals_list: Vec<ObjectDescription> = Vec::default();
    let mut constants_list: Vec<ObjectDescription> = Vec::default();

    // loop through architectures
    for (tokens, design_unit) in &design.design_units {
        if let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(architecture)) = design_unit {
            
            // loop through declarations in the architecture
            for declaration in &architecture.decl {

                if let Object(object_declaration) = &declaration.item {

                    // extract details about the objects
                    let object_type = object_declaration.subtype_indication.to_string();
                    let expression = object_declaration.expression.as_ref().map(|e| e.to_string());
                    let description = find_object_description(tokens, declaration.get_start_token(),priority_trailing, false);

                    // creates one ObjectDescription for each object
                    let mut objects_list: Vec<ObjectDescription> = object_declaration.idents.iter().map(|ident|

                        ObjectDescription {
                            name: ident.tree.item.to_string(),
                            object_type: object_type.clone(),
                            expression: expression.clone(),
                            description: description.clone()
                        }
                    ).collect();

                    // add the list to the correct list, depending on whether it's a signal or a constant
                    match object_declaration.class {
                        Signal => {
                            signals_list.append(&mut objects_list);
                        },
                        Constant => {
                            constants_list.append(&mut objects_list);
                        },
                        _ => {
                        }
                    }
                }
            }
        }
    }

    Ok(SignalsConstants {
        signals: signals_list,
        constants: constants_list
    }) 
}
