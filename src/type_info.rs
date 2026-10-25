use vhdl_lang::ast::{Direction, DiscreteRange, Range, SubtypeConstraint};

/// extracted information about a (sub)type
pub struct TypeInfo {
    /// the name for the VHDL type
    type_name: String,
    /// The constraint ( x downto y )
    constraint: Option<String>,
    /// The size (as an array of string, one item per dimension)
    type_sizes: Vec<String>,
}

/// get the size from a range object
pub fn range_to_size(range: &Range) -> String {
    match range {
        Range::Attribute(_) => "".to_owned(),
        Range::Range(range) => {
            let expr = match range.direction {
                Direction::Ascending => {
                    format!("({})-({})+1", range.right_expr, range.left_expr)
                }
                Direction::Descending => {
                    format!("({})-({})+1", range.left_expr, range.right_expr)
                }
            };

            // attempt to simplify the expression
            match arael_sym::parse(&expr) {
                Ok(parsed_expr) => parsed_expr.simplify().to_string(),
                Err(_) => expr,
            }
        }
    }
}

impl TypeInfo {
    /// extract the type information from a SubTypeIndication object
    pub fn new(fromtype: &vhdl_lang::ast::SubtypeIndication) -> Self {
        let type_name = fromtype.type_mark.to_string();
        let constraint = fromtype
            .constraint
            .as_ref()
            .map(|constraint| constraint.to_string());
        let type_sizes = match &fromtype.constraint {
            Some(fromtype_constraint) => {
                match &fromtype_constraint.item {
                    SubtypeConstraint::Range(r) => {
                        // single range. Just convert it to size
                        vec![range_to_size(&r)]
                    }
                    SubtypeConstraint::Array(a, _) => {
                        // array with several dimensions. Go through all of them
                        a.iter()
                            .map(|x| match &x.item {
                                DiscreteRange::Discrete(_, range_option) => match range_option {
                                    None => String::new(),
                                    Some(r) => range_to_size(&r),
                                },
                                DiscreteRange::Range(r) => range_to_size(&r),
                            })
                            .collect()
                    }
                    SubtypeConstraint::Record(_) => Vec::new(),
                }
            }
            None => Vec::new(),
        };

        TypeInfo {
            type_name,
            constraint,
            type_sizes,
        }
    }

    /// fill with ready parsed information
    pub fn fill(type_name: String, constraint: Option<String>, type_sizes: Vec<String>) -> Self {
        TypeInfo {
            type_name,
            constraint,
            type_sizes,
        }
    }

    /// return a copy of the type name
    pub fn get_type_name(&self) -> String {
        self.type_name.clone()
    }

    /// return a copy of the constraint
    pub fn get_constraint(&self) -> Option<String> {
        self.constraint.clone()
    }

    /// return a copy of the sizes table
    pub fn get_sizes(&self) -> Vec<String> {
        self.type_sizes.clone()
    }
}
