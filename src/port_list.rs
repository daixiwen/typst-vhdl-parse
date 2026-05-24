use vhdl_lang::ast::DesignFile;
use vhdl_lang::ast::{AnyDesignUnit, AnyPrimaryUnit, InterfaceDeclaration, ModeIndication};

pub struct PortEntry {
    pub name: String,
    pub mode: String,
    pub port_type: String,
    pub constraint: String,
}

pub fn get_port_list(design: DesignFile) -> Result<Vec<PortEntry>, String> {
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
                            entries.push(PortEntry {
                                name: name,
                                mode: mode_str.clone(),
                                port_type: type_str.clone(),
                                constraint: constraint_str.clone(),
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
                            entries.push(PortEntry {
                                name: name,
                                mode: "file".to_owned(),
                                port_type: typ.clone(),
                                constraint: String::new(),
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
