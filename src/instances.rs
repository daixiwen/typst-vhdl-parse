use serde::Serialize;
use vhdl_lang::ast::ConcurrentStatement::{Block, CaseGenerate, ForGenerate, IfGenerate, Instance};
use vhdl_lang::ast::token_range::WithTokenSpan;
use vhdl_lang::ast::{
    AnyDesignUnit, AnySecondaryUnit, DesignFile, Ident, InstantiatedUnit, InstantiationStatement,
    LabeledConcurrentStatement, MapAspect, Name, WithDecl,
};
use vhdl_lang::{HasTokenSpan, Token};

use crate::comments::find_object_description;

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
pub fn get_instances(design: DesignFile) -> Result<Vec<InstanceDescription>, String> {
    let mut instances_list: Vec<InstanceDescription> = Vec::default();

    for (tokens, design_unit) in &design.design_units {
        // look for an architecture, and more precisely the body part
        if let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(architecture)) = design_unit
        {
            // loop through the concurrent statements in the body. Put the result in fsm_description
            process_concurrent_statements(tokens, &architecture.statements, &mut instances_list)?;
        };
    }

    Ok(instances_list)
}

/// go through concurrent statements, looking for a process
fn process_concurrent_statements(
    tokens: &Vec<Token>,
    statements: &Vec<LabeledConcurrentStatement>,
    instances_list: &mut Vec<InstanceDescription>,
) -> Result<(), String> {
    for statement in statements {
        // for every concurrent statement holding other concurrent statements, go through them
        match &statement.statement.item {
            Block(block_statement) => {
                process_concurrent_statements(tokens, &block_statement.statements, instances_list)?;
            }

            ForGenerate(for_generate_statement) => {
                process_concurrent_statements(
                    tokens,
                    &for_generate_statement.body.statements,
                    instances_list,
                )?;
            }

            IfGenerate(if_generate_statement) => {
                for conditional in &if_generate_statement.conds.conditionals {
                    process_concurrent_statements(
                        tokens,
                        &conditional.item.statements,
                        instances_list,
                    )?;
                }
                if let Some(body) = &if_generate_statement.conds.else_item {
                    process_concurrent_statements(tokens, &body.0.statements, instances_list)?;
                }
            }

            CaseGenerate(case_generate_statement) => {
                for alternative in &case_generate_statement.sels.alternatives {
                    process_concurrent_statements(
                        tokens,
                        &alternative.item.statements,
                        instances_list,
                    )?;
                }
            }

            // we found an instantiation. Decode it and add it to the list
            Instance(instance_statement) => {
                instances_list.push(find_instance(tokens, &statement.label, instance_statement)?);
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
) -> Result<InstanceDescription, String> {
    match &instance_statement.unit {
        InstantiatedUnit::Component(component) => {
            decode_instance(tokens, label, instance_statement, component)
        }
        InstantiatedUnit::Entity(entity, _) => {
            decode_instance(tokens, label, instance_statement, entity)
        }
        InstantiatedUnit::Configuration(config) => {
            decode_instance(tokens, label, instance_statement, config)
        }
    }
}

/// find all the required information from the instance statement
fn decode_instance(
    tokens: &Vec<Token>,
    label_decl: &WithDecl<Option<Ident>>,
    instance_statement: &InstantiationStatement,
    name: &WithTokenSpan<Name>,
) -> Result<InstanceDescription, String> {
    let label = match &label_decl.tree {
        Some(item) => item.to_string(),
        None => String::new(),
    };
    let entity = name.item.to_string();
    let description =
        find_object_description(tokens, instance_statement.get_start_token(), false, false);

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
