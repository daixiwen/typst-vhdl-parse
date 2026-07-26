use serde::Serialize;
use vhdl_lang::ast::ConcurrentStatement::{Block, CaseGenerate, ForGenerate, IfGenerate, Instance};
use vhdl_lang::ast::token_range::WithTokenSpan;
use vhdl_lang::ast::{
    AnyDesignUnit, AnySecondaryUnit, DesignFile, Ident, InstantiatedUnit, InstantiationStatement,
    LabeledConcurrentStatement, MapAspect, Name, WithDecl,
};
use vhdl_lang::{HasTokenSpan, Token};

use crate::comments::find_object_description;
use crate::parse_store::get_parsed;
use crate::{decode_typst_arg, decode_typst_arg_id, encode_typst_return};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

/// Describes a single instantiation
#[derive(Serialize)]
pub struct InstanceDescription {
    /// label for the instance
    pub label: String,
    /// entity being instanciated
    pub entity: String,
    /// a description of the instantiation
    pub description: Option<String>,
    /// a list of generic assignements
    pub generics_map: Vec<InstanceAssignment>,
    /// a list of port associations
    pub ports_map: Vec<InstanceAssignment>,
}

#[derive(Serialize)]
/// Describes an assignment to a generic or a port
pub struct InstanceAssignment {
    /// origin: generic or port being mapped
    pub origin: String,
    /// expression it is being mapped to
    pub expression: String,
}

/// return the list of instances in a design file
pub fn get_instances(
    design: DesignFile,
    priority_trailing: bool,
) -> Result<Vec<InstanceDescription>, String> {
    let mut instances_list: Vec<InstanceDescription> = Vec::default();

    for (tokens, design_unit) in &design.design_units {
        // look for an architecture, and more precisely the body part
        if let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(architecture)) = design_unit
        {
            // loop through the concurrent statements in the body. Put the result in fsm_description
            process_concurrent_statements(
                tokens,
                &architecture.statements,
                &mut instances_list,
                priority_trailing,
            )?;
        };
    }

    Ok(instances_list)
}

/// go through concurrent statements, looking for a process
fn process_concurrent_statements(
    tokens: &Vec<Token>,
    statements: &Vec<LabeledConcurrentStatement>,
    instances_list: &mut Vec<InstanceDescription>,
    priority_trailing: bool,
) -> Result<(), String> {
    for statement in statements {
        // for every concurrent statement holding other concurrent statements, go through them
        match &statement.statement.item {
            Block(block_statement) => {
                process_concurrent_statements(
                    tokens,
                    &block_statement.statements,
                    instances_list,
                    priority_trailing,
                )?;
            }

            ForGenerate(for_generate_statement) => {
                process_concurrent_statements(
                    tokens,
                    &for_generate_statement.body.statements,
                    instances_list,
                    priority_trailing,
                )?;
            }

            IfGenerate(if_generate_statement) => {
                for conditional in &if_generate_statement.conds.conditionals {
                    process_concurrent_statements(
                        tokens,
                        &conditional.item.statements,
                        instances_list,
                        priority_trailing,
                    )?;
                }
                if let Some(body) = &if_generate_statement.conds.else_item {
                    process_concurrent_statements(
                        tokens,
                        &body.0.statements,
                        instances_list,
                        priority_trailing,
                    )?;
                }
            }

            CaseGenerate(case_generate_statement) => {
                for alternative in &case_generate_statement.sels.alternatives {
                    process_concurrent_statements(
                        tokens,
                        &alternative.item.statements,
                        instances_list,
                        priority_trailing,
                    )?;
                }
            }

            // we found an instantiation. Decode it and add it to the list
            Instance(instance_statement) => {
                instances_list.push(find_instance(
                    tokens,
                    &statement.label,
                    instance_statement,
                    priority_trailing,
                )?);
            }

            _ => {}
        }
    }
    Ok(())
}

/// find the kind of instance and call the decode function with the correct parameters
fn find_instance(
    tokens: &Vec<Token>,
    label: &WithDecl<Option<Ident>>,
    instance_statement: &InstantiationStatement,
    priority_trailing: bool,
) -> Result<InstanceDescription, String> {
    match &instance_statement.unit {
        InstantiatedUnit::Component(component) => decode_instance(
            tokens,
            label,
            instance_statement,
            component,
            priority_trailing,
        ),
        InstantiatedUnit::Entity(entity, _) => {
            decode_instance(tokens, label, instance_statement, entity, priority_trailing)
        }
        InstantiatedUnit::Configuration(config) => {
            decode_instance(tokens, label, instance_statement, config, priority_trailing)
        }
    }
}

/// find all the required information from the instance statement
fn decode_instance(
    tokens: &Vec<Token>,
    label_decl: &WithDecl<Option<Ident>>,
    instance_statement: &InstantiationStatement,
    name: &WithTokenSpan<Name>,
    priority_trailing: bool,
) -> Result<InstanceDescription, String> {
    let label = match &label_decl.tree {
        Some(item) => item.to_string(),
        None => String::new(),
    };
    let entity = name.item.to_string();
    let description = find_object_description(
        tokens,
        instance_statement.get_start_token(),
        priority_trailing,
        false,
    );

    let generics_map = get_map(&instance_statement.generic_map);
    let ports_map = get_map(&instance_statement.port_map);

    Ok(InstanceDescription {
        label,
        entity,
        description,
        generics_map,
        ports_map,
    })
}

/// find all the associations in a generic or a ports map
fn get_map(map: &Option<MapAspect>) -> Vec<InstanceAssignment> {
    match map {
        Some(map) => {
            // go through each item of the map and generate an InstanceAssignment structure with the required elements
            map.list
                .items
                .iter()
                .map(|item| {
                    let origin = match &item.formal {
                        Some(formal) => formal.item.to_string(),
                        None => String::new(),
                    };
                    let expression = item.actual.to_string();

                    InstanceAssignment { origin, expression }
                })
                .collect()
        }
        None => Vec::new(),
    }
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
    fn test_instances() {
        let design = parse_test_file();
        let instances = get_instances(design, false).unwrap();

        // check the returned list
        assert_eq!(instances.len(), 1);

        // check the first instance
        let my_instance = instances.get(0).unwrap();
        assert_eq!(my_instance.label, "my_instance");
        assert_eq!(my_instance.entity, "work.comp");
        assert_eq!(
            my_instance.description,
            Some("an example instantiation".to_owned())
        );

        // check the generic map
        assert_eq!(my_instance.generics_map.len(), 1);
        let generic = my_instance.generics_map.get(0).unwrap();
        assert_eq!(generic.origin, "enabled");
        assert_eq!(generic.expression, "true");

        // check the port map
        assert_eq!(my_instance.ports_map.len(), 4);
        let port = my_instance.ports_map.get(0).unwrap();
        assert_eq!(port.origin, "clk");
        assert_eq!(port.expression, "clock");
        let port = my_instance.ports_map.get(1).unwrap();
        assert_eq!(port.origin, "rst");
        assert_eq!(port.expression, "reset");
        let port = my_instance.ports_map.get(2).unwrap();
        assert_eq!(port.origin, "input");
        assert_eq!(port.expression, "input_b");
        let port = my_instance.ports_map.get(3).unwrap();
        assert_eq!(port.origin, "output");
        assert_eq!(port.expression, "output_a");
    }
}

/// typst plugin function to find the instances in a file and return it as an array of InstanceDescription
#[allow(dead_code)]
#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_instances_list(
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

    let instances = get_instances(designfile, priority_trailing)?;

    encode_typst_return(&instances)
}
