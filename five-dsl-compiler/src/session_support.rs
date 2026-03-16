use crate::ast::{AstNode, Attribute, InstructionParameter, TypeNode};

pub const IMPLICIT_SESSION_PARAM_NAME: &str = "__session";

pub const SESSION_V1_FIELDS: [&str; 12] = [
    "authority",
    "delegate",
    "target_program",
    "expires_at_slot",
    "scope_hash",
    "nonce",
    "bind_account",
    "manager_script_account",
    "manager_code_hash",
    "manager_version",
    "status",
    "version",
];

pub fn session_deprecation_warnings_enabled() -> bool {
    match std::env::var("FIVE_SUPPRESS_SESSION_DEPRECATION_WARNINGS") {
        Ok(value) => {
            let lowered = value.to_ascii_lowercase();
            !(lowered == "1" || lowered == "true" || lowered == "yes")
        }
        Err(_) => true,
    }
}

pub fn is_session_type(param: &InstructionParameter) -> bool {
    match &param.param_type {
        TypeNode::Named(name) => name == "Session" || name.ends_with("::Session"),
        _ => false,
    }
}

pub fn find_session_attribute(param: &InstructionParameter) -> Option<&Attribute> {
    param.attributes.iter().find(|attr| attr.name == "session")
}

pub fn has_keyed_session_args(attribute: &Attribute) -> bool {
    attribute
        .args
        .iter()
        .any(|arg| matches!(arg, AstNode::Assignment { .. }))
}

fn attr_value<'a>(attribute: &'a Attribute, key: &str) -> Option<&'a AstNode> {
    for arg in &attribute.args {
        if let AstNode::Assignment { target, value } = arg {
            if target == key {
                return Some(value.as_ref());
            }
        }
    }
    None
}

fn session_arg<'a>(attribute: &'a Attribute, key: &str, pos: usize) -> Option<&'a AstNode> {
    if has_keyed_session_args(attribute) {
        return attr_value(attribute, key);
    }
    attribute.args.get(pos)
}

pub fn inject_implicit_session_param(parameters: &[InstructionParameter]) -> Vec<InstructionParameter> {
    let mut authority_param_name: Option<String> = None;
    let mut source_session_attr: Option<Attribute> = None;
    let mut transformed: Vec<InstructionParameter> = Vec::with_capacity(parameters.len() + 1);

    for param in parameters {
        let mut new_param = param.clone();
        let mut retained_attrs = Vec::with_capacity(new_param.attributes.len());
        for attr in &new_param.attributes {
            if attr.name == "session" {
                authority_param_name = Some(new_param.name.clone());
                source_session_attr = Some(attr.clone());
                continue;
            }
            retained_attrs.push(attr.clone());
        }
        // @session implies signer semantics on the authority/owner account.
        if authority_param_name.as_deref() == Some(new_param.name.as_str())
            && !retained_attrs.iter().any(|attr| attr.name == "signer")
        {
            retained_attrs.push(Attribute {
                name: "signer".to_string(),
                args: vec![],
            });
        }
        new_param.attributes = retained_attrs;
        transformed.push(new_param);
    }

    let (Some(authority_name), Some(attribute)) = (authority_param_name, source_session_attr) else {
        return parameters.to_vec();
    };

    let mut args: Vec<AstNode> = Vec::new();
    for (key, pos) in [
        ("authority", 0usize),
        ("target_program", 1usize),
        ("scope_hash", 2usize),
        ("bind_account", 3usize),
        ("nonce_field", 4usize),
        ("current_slot", 5usize),
        ("manager_script_account", 6usize),
        ("manager_code_hash", 7usize),
        ("manager_version", 8usize),
    ] {
        if let Some(value) = session_arg(&attribute, key, pos) {
            args.push(AstNode::Assignment {
                target: key.to_string(),
                value: Box::new(value.clone()),
            });
        }
    }
    if attr_value(&attribute, "authority").is_none() {
        args.push(AstNode::Assignment {
            target: "authority".to_string(),
            value: Box::new(AstNode::Identifier(authority_name)),
        });
    }

    transformed.push(InstructionParameter {
        name: IMPLICIT_SESSION_PARAM_NAME.to_string(),
        param_type: TypeNode::Named("Session".to_string()),
        is_optional: false,
        default_value: None,
        attributes: vec![Attribute {
            name: "session".to_string(),
            args,
        }],
        is_init: false,
        init_config: None,
        serializer: None,
        pda_config: None,
    });

    transformed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account_param(name: &str, attrs: Vec<Attribute>) -> InstructionParameter {
        InstructionParameter {
            name: name.to_string(),
            param_type: TypeNode::Account,
            is_optional: false,
            default_value: None,
            attributes: attrs,
            is_init: false,
            init_config: None,
            serializer: None,
            pda_config: None,
        }
    }

    #[test]
    fn injects_hidden_session_for_authority_attached_form() {
        let params = vec![
            account_param("authority", vec![Attribute {
                name: "session".to_string(),
                args: vec![
                    AstNode::Assignment {
                        target: "delegate".to_string(),
                        value: Box::new(AstNode::Identifier("delegate".to_string())),
                    },
                    AstNode::Assignment {
                        target: "nonce_field".to_string(),
                        value: Box::new(AstNode::Identifier("nonce".to_string())),
                    },
                ],
            }]),
            account_param(
                "delegate",
                vec![Attribute {
                    name: "signer".to_string(),
                    args: vec![],
                }],
            ),
        ];
        let effective = inject_implicit_session_param(&params);
        assert_eq!(effective.len(), 3);
        assert!(effective
            .iter()
            .any(|param| param.name == IMPLICIT_SESSION_PARAM_NAME));
        assert!(!effective.iter().any(|param| param.name == "__delegate"));
        let authority = effective.iter().find(|param| param.name == "authority").unwrap();
        assert!(!authority.attributes.iter().any(|attr| attr.name == "session"));
        assert!(authority.attributes.iter().any(|attr| attr.name == "signer"));
    }
}
