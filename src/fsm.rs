use serde::{Deserialize, Serialize};
use std::default::Default;
use vhdl_lang::{HasTokenSpan, Token, TokenId};
use vhdl_lang::ast::ConcurrentStatement::{Block, CaseGenerate, ForGenerate, IfGenerate, Process};
use vhdl_lang::ast::Designator::Identifier;
use vhdl_lang::ast::SequentialStatement::{Case, If, Loop, SignalAssignment, VariableAssignment};
use vhdl_lang::ast::Waveform::Elements;
use vhdl_lang::ast::{AnyDesignUnit, AnySecondaryUnit, AssignmentRightHand, Choice, Name, Target};
use vhdl_lang::ast::{DesignFile, LabeledConcurrentStatement, LabeledSequentialStatement};

use crate::parse_store::get_parsed;
use crate::{decode_typst_arg_struct, decode_typst_arg_id, encode_typst_return};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[derive(Serialize, Default, Debug)]
pub struct FSMDescription {
    pub default_state: String,
    pub states: Vec<FSMState>,
}

#[derive(Serialize, Debug)]
pub struct FSMState {
    pub name: String,
    pub description: Option<String>,
    pub transitions: Vec<FSMTransition>,
}

#[derive(Serialize, Clone, Debug)]
pub struct FSMTransition {
    pub destination: String,
    pub condition: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct FSMConfig {
    pub read_variable_name: String,
    pub write_variable_name: String,
}

// look for a state machine in a design file
pub fn get_fsm(design: DesignFile, config: &FSMConfig) -> Result<FSMDescription, String> {
    let mut fsm_description = FSMDescription::default();

    for (tokens, design_unit) in &design.design_units {
        if let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(architecture)) = design_unit
        {
            process_concurrent_statements(
                tokens,
                &architecture.statements,
                config,
                &mut fsm_description,
            );
        };
    }

    if fsm_description.states.len() == 0 {
        Err("state machine not found".to_owned())
    } else {
        Ok(fsm_description)
    }
}

// go through concurrent statements, looking for a process
fn process_concurrent_statements(
    tokens: &Vec<Token>,
    statements: &Vec<LabeledConcurrentStatement>,
    config: &FSMConfig,
    fsm_description: &mut FSMDescription,
) {
    for statement in statements {
        match &statement.statement.item {
            // for every concurrent statement holding other concurrent statements, go through them
            Block(block_statement) => {
                process_concurrent_statements(
                    tokens,
                    &block_statement.statements,
                    config,
                    fsm_description,
                );
            }

            ForGenerate(for_generate_statement) => {
                process_concurrent_statements(
                    tokens,
                    &for_generate_statement.body.statements,
                    config,
                    fsm_description,
                );
            }

            IfGenerate(if_generate_statement) => {
                for conditional in &if_generate_statement.conds.conditionals {
                    process_concurrent_statements(
                        tokens,
                        &conditional.item.statements,
                        config,
                        fsm_description,
                    );
                }
                if let Some(body) = &if_generate_statement.conds.else_item {
                    process_concurrent_statements(
                        tokens,
                        &body.0.statements,
                        config,
                        fsm_description,
                    );
                }
            }

            CaseGenerate(case_generate_statement) => {
                for alternative in &case_generate_statement.sels.alternatives {
                    process_concurrent_statements(
                        tokens,
                        &alternative.item.statements,
                        config,
                        fsm_description,
                    );
                }
            }

            // we found a process. We can go through its sequential lines
            Process(process_statement) => {
                find_case(
                    tokens,
                    &process_statement.statements,
                    config,
                    fsm_description,
                );
            }

            _ => {}
        }
    }
}

// go through sequential statements, looking for a FSM in a case statement
fn find_case(
    tokens: &Vec<Token>,
    statements: &Vec<LabeledSequentialStatement>,
    config: &FSMConfig,
    fsm_description: &mut FSMDescription,
) {
    for statement in statements {
        match &statement.statement.item {
            Case(case_statement) => {
                if case_statement.expression.to_string() == config.read_variable_name {
                    // we found the case with the state machine we are looking for....

                    // go through each case
                    for alternative in &case_statement.alternatives {
                        // look for transitions
                        let mut transitions: Vec<FSMTransition> = Vec::new();
                        find_transitions(
                            tokens,
                            &alternative.item,
                            config,
                            "".to_owned(),
                            alternative.item.first().map(|statement| statement.get_start_token()),
                            &mut transitions,
                        );

                        let description = crate::comments::find_object_description(tokens, alternative.get_start_token(), true, false);

                        // go through the choices
                        for choice in &alternative.choices {
                            match &choice.item {
                                Choice::Expression(expression) => {
                                    fsm_description.states.push(FSMState {
                                        name: expression.to_string(),
                                        description: description.clone(),
                                        transitions: transitions.clone(),
                                    })
                                }
                                Choice::Others => {
                                    // there shouldn't be anything more than a jump to the reset state in here
                                    if let Some(transition) = transitions.get(0) {
                                        fsm_description.default_state = transition.destination.clone();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    // this is not the state machine, but the state machine could be in one of the cases
                    for alternative in &case_statement.alternatives {
                        find_case(tokens, &alternative.item, config, fsm_description);
                    }
                }
            },
            If(if_statement) => {
                // explore every branch

                for condition in &if_statement.conds.conditionals {
                    find_case(tokens,
                        &condition.item,
                        config,
                        fsm_description);
                }
                if let Some(else_condition) = &if_statement.conds.else_item {
                    find_case(tokens,
                        &else_condition.0,
                        config,
                        fsm_description);
                }
            },
            Loop(loop_statement) => {
                // explore inside the loop

                find_case(tokens, &loop_statement.statements, config, fsm_description);
            },
            _ => {
            }
        }
    }
}

// go through sequential statements and look for transitions
fn find_transitions(
    tokens: &Vec<Token>,
    statements: &Vec<LabeledSequentialStatement>,
    config: &FSMConfig,
    condition: String,
    condition_token: Option<TokenId>,
    transitions: &mut Vec<FSMTransition>,
) {
    for statement in statements {
        match &statement.statement.item {
            VariableAssignment(variable_assignment) => {
                if let Target::Name(Name::Designator(name_designator)) =
                    &variable_assignment.target.item
                {
                    if let Identifier(symbol) = &name_designator.item {
                        if symbol.name_utf8() == config.write_variable_name {
                            // we are assigning to the correct variable
                            if let AssignmentRightHand::Simple(expression) =
                                &variable_assignment.rhs
                            {
                                let target = expression.item.to_string();

                                let description = if let Some(tokenid) = condition_token {
                                    crate::comments::find_object_description(tokens, tokenid, true, false)
                                } else {
                                    None
                                };

                                transitions.push(FSMTransition {
                                    destination: target,
                                    condition: condition.clone(),
                                    description: description,
                                });
                            }
                        }
                    }
                }
            }

            SignalAssignment(signal_assignment) => {
                if let Target::Name(Name::Designator(name_designator)) =
                    &signal_assignment.target.item
                {
                    if let Identifier(symbol) = &name_designator.item {
                        if symbol.name_utf8() == config.write_variable_name {
                            // we are assigning to the correct variable
                            if let AssignmentRightHand::Simple(Elements(elements)) =
                                &signal_assignment.rhs
                            {
                                if let Some(element) = elements.get(0) {
                                    // there shouldn't be more than one waveform in synthesized VHDL. We'll read only the first one
                                    let target = element.value.item.to_string();

                                    let description = if let Some(tokenid) = condition_token {
                                        crate::comments::find_object_description(tokens, tokenid, true, false)
                                    } else {
                                        None
                                    };

                                    transitions.push(FSMTransition {
                                        destination: target,
                                        condition: condition.clone(),
                                        description: description,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            If(if_statement) => {
                // go through each if/elsif
                for conditional in &if_statement.conds.conditionals {
                    find_transitions(
                        tokens,
                        &conditional.item,
                        config,
                        conditional.condition.item.to_string(),
                        Some(conditional.condition.get_start_token()),
                        transitions,
                    );
                }
                // and the else branch
                if let Some(else_statements) = &if_statement.conds.else_item {
                    find_transitions(
                        tokens,
                        &else_statements.0,
                        config,
                        condition.clone(),
                        Some(else_statements.1),
                        transitions,
                    );
                }
            }

            Case(case_statement) => {
                // generate the beginning of the condition for each case
                let condition_begin = case_statement.expression.item.to_string();

                for alternative in &case_statement.alternatives {
                    // build the condition as a string
                    let choices: Vec<String> = alternative
                        .choices
                        .iter()
                        .map(|c| c.item.to_string())
                        .collect();
                    let case_condition = format!("{} = {}", condition_begin, choices.join(" | "));

                    // go through the statements
                    find_transitions(
                        tokens,
                        &alternative.item,
                        config,
                        case_condition,
                        Some(alternative.get_start_token()),
                        transitions,
                    );
                }
            }

            Loop(loop_statement) => {
                // in a loop, just go through the inner statements
                find_transitions(
                    tokens,
                    &loop_statement.statements,
                    config,
                    condition.clone(),
                    condition_token,
                    transitions,
                );
            }
            _ => {}
        }
    }
}

impl FSMDescription {

    /// generate a dot description of the fsm
    pub fn to_dot(&self) -> Result<String,String> {
        let mut result = string_builder::Builder::default();

        result.append("digraph {\n");

        if self.default_state.len() > 0 {
            result.append(format!("  node [shape=doublecircle]\n  {}\n  node [shape=circle]\n", self.default_state));
        }

        for state in &self.states {
            for transition in &state.transitions {
                if transition.condition.len() > 0 {
                    result.append(format!(
                        "  {} -> {}[label=\"{}\"]\n",
                        state.name,
                        transition.destination,
                        transition.condition));
                } else {
                    result.append(format!(
                        "  {} -> {}\n",
                        state.name,
                        transition.destination));
                }
            }
        }
        result.append("}\n");

        result.string().map_err(|e| e.to_string())
    }
}


#[cfg_attr(target_arch = "wasm32", wasm_func)]
fn get_fsm_as_dot(id: &[u8], config_str: &[u8]) -> Result<Vec<u8>, String> {
    let id = decode_typst_arg_id(id)?;
    let config : FSMConfig = decode_typst_arg_struct(config_str)?;

    let designfile = get_parsed(id)?;

    let fsm = get_fsm(designfile, &config)?;

    encode_typst_return(&fsm.to_dot()?)
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
    fn test_fsm_description() {
        let design = parse_test_file();
        let fsm = get_fsm(
            design,
            &FSMConfig {
                read_variable_name: "fsm".to_owned(),
                write_variable_name: "fsm".to_owned(),
            },
        )
        .unwrap();

        // check the returned structure
        assert_eq!(fsm.default_state, "reset");
        assert_eq!(fsm.states.len(), 4);
        assert_eq!(fsm.states[0].name, "reset");
        assert_eq!(fsm.states[0].description, Some("start and reset state".to_owned()));
        assert_eq!(fsm.states[0].transitions.len(), 1);
        assert_eq!(fsm.states[0].transitions[0].destination, "idle");
        assert_eq!(fsm.states[0].transitions[0].condition, "");
        assert_eq!(fsm.states[0].transitions[0].description, Some("out of reset".to_owned()));
        assert_eq!(fsm.states[1].name, "idle");
        assert_eq!(fsm.states[1].description, Some("normal state when nothing happens".to_owned()));
        assert_eq!(fsm.states[1].transitions.len(), 1);
        assert_eq!(fsm.states[1].transitions[0].destination, "read_input");
        assert_eq!(fsm.states[1].transitions[0].condition, "input_b = '1'");
        assert_eq!(fsm.states[1].transitions[0].description, Some("new input".to_owned()));
        assert_eq!(fsm.states[2].name, "read_input");
        assert_eq!(fsm.states[2].description, Some("waiting for input".to_owned()));
        assert_eq!(fsm.states[2].transitions.len(), 2);
        assert_eq!(fsm.states[2].transitions[0].destination, "idle");
        assert_eq!(fsm.states[2].transitions[0].condition, "input_d = '1'");
        assert_eq!(fsm.states[2].transitions[0].description, None);
        assert_eq!(fsm.states[2].transitions[1].destination, "write_output");
        assert_eq!(fsm.states[2].transitions[1].condition, "data_in = std_logic_vector(value)");
        assert_eq!(fsm.states[2].transitions[1].description, Some("correct input".to_owned()));
        assert_eq!(fsm.states[3].name, "write_output");
        assert_eq!(fsm.states[3].description, Some("send output".to_owned()));
        assert_eq!(fsm.states[3].transitions.len(), 2);
        assert_eq!(fsm.states[3].transitions[0].destination, "idle");
        assert_eq!(fsm.states[3].transitions[0].condition, "input_b = '0'");
        assert_eq!(fsm.states[3].transitions[0].description, None);
        assert_eq!(fsm.states[3].transitions[1].destination, "write_output");
        assert_eq!(fsm.states[3].transitions[1].condition, "");
        assert_eq!(fsm.states[3].transitions[1].description, Some("stay".to_owned()));

        println!("dot:\n{}",fsm.to_dot().unwrap());
    }
}
