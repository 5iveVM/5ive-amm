// Account type detection helpers.

use super::types::AccountRegistry;
use crate::ast::{Attribute, TypeNode};

/// Map a 0-based parameter index to the absolute account index used on-chain.
pub fn account_index_from_param_index(param_index: u8) -> u8 {
    let offset = super::ACCOUNT_INDEX_OFFSET;
    let result = param_index.saturating_add(offset);
    println!(
        "DEBUG: account_index_from_param_index({}) with offset {} -> {}",
        param_index, offset, result
    );
    result
}

/// Map a parameter offset stored in symbol tables (0-based) to an account index.
pub fn account_index_from_param_offset(offset: u32) -> u8 {
    account_index_from_param_index(offset as u8)
}

/// Unified account type detection used across compiler modules.
pub fn is_account_type(type_node: &TypeNode, account_registry: Option<&AccountRegistry>) -> bool {
    match type_node {
        TypeNode::Named(name) => {
            // First check the account registry for custom defined types
            if let Some(registry) = account_registry {
                if registry.account_types.contains_key(name) {
                    return true;
                }

                let tail = name.rsplit("::").next().unwrap_or(name);
                if registry
                    .account_types
                    .keys()
                    .any(|key| key == name || key.rsplit("::").next() == Some(tail))
                {
                    return true;
                }
            }

            // Check built-in account types
            if matches!(
                name.as_str(),
                "Account" | "TokenAccount" | "ProgramAccount" | "account"
            ) {
                return true;
            }

            // Pattern-based detection: types ending with "Account"
            // This catches custom types like StateAccount, VaultAccount, UserAccount, etc.
            name.ends_with("Account")
        }

        TypeNode::Primitive(name) => {
            // Handle primitive account types
            matches!(
                name.as_str(),
                "Account" | "TokenAccount" | "ProgramAccount" | "account"
            )
        }

        TypeNode::Generic { base, .. } => {
            // Handle generic account types like Account<T>
            matches!(
                base.as_str(),
                "Account" | "TokenAccount" | "ProgramAccount" | "account"
            ) || base.ends_with("Account")
        }

        TypeNode::Account => true,

        _ => false,
    }
}

/// Check if a parameter has account-related attributes
///
/// Account parameters often have attributes like @mut, @signer, @init that indicate
/// they are accounts even if the type name doesn't follow conventions
pub fn has_account_attributes(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attr| {
        matches!(
            attr.name.as_str(),
            "mut" | "signer" | "init" | "writable" | "owner" | "close"
        )
    })
}

/// Enhanced account type detection that also considers parameter attributes
///
/// This function combines type-based detection with attribute analysis for
/// comprehensive account parameter identification
/// Enhanced account type detection that also considers parameter attributes
///
/// This function combines type-based detection with attribute analysis for
/// comprehensive account parameter identification
pub fn is_account_parameter(
    type_node: &TypeNode,
    attributes: &[Attribute],
    account_registry: Option<&AccountRegistry>,
) -> bool {
    // panic!("I AM NEW VERSION - DEBUGGING");
    // println!("DEBUG: checking is_account_parameter for type '{:?}' attrs '{:?}'", type_node, attributes);

    // Primary check: type-based detection
    if is_account_type(type_node, account_registry) {
        println!("DEBUG: type-based detection passed");
        return true;
    }

    // Secondary check: attribute-based detection
    // If a parameter has account attributes, it's likely an account
    let has_attrs = has_account_attributes(attributes);
    println!("DEBUG: attribute-based detection: {}", has_attrs);
    has_attrs
}

/// Convert TypeNode to string for debugging and display
pub fn type_node_to_string(type_node: &TypeNode) -> String {
    match type_node {
        TypeNode::Primitive(name) => name.clone(),
        TypeNode::Named(name) => name.clone(),
        TypeNode::Generic { base, args } => {
            if args.is_empty() {
                base.clone()
            } else {
                let arg_strings: Vec<String> = args.iter().map(type_node_to_string).collect();
                format!("{}<{}>", base, arg_strings.join(", "))
            }
        }
        TypeNode::Array { element_type, size } => {
            if let Some(size) = size {
                format!("[{}; {}]", type_node_to_string(element_type), size)
            } else {
                format!("[{}]", type_node_to_string(element_type))
            }
        }
        TypeNode::Tuple { elements } => {
            let type_strings: Vec<String> = elements.iter().map(type_node_to_string).collect();
            format!("({})", type_strings.join(", "))
        }
        TypeNode::Struct { fields } => {
            format!("struct{{{}}}", fields.len())
        }
        TypeNode::Union { types } => {
            let type_strings: Vec<String> = types.iter().map(type_node_to_string).collect();
            type_strings.join(" | ")
        }
        TypeNode::Sized { base_type, size } => {
            format!("{}<{}>", base_type, size)
        }
        TypeNode::Account => "Account".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_account_types() {
        assert!(is_account_type(
            &TypeNode::Named("Account".to_string()),
            None
        ));
        assert!(is_account_type(
            &TypeNode::Named("TokenAccount".to_string()),
            None
        ));
        assert!(is_account_type(
            &TypeNode::Named("ProgramAccount".to_string()),
            None
        ));
        assert!(is_account_type(
            &TypeNode::Primitive("Account".to_string()),
            None
        ));
    }

    #[test]
    fn test_custom_account_types() {
        assert!(is_account_type(
            &TypeNode::Named("StateAccount".to_string()),
            None
        ));
        assert!(is_account_type(
            &TypeNode::Named("VaultAccount".to_string()),
            None
        ));
        assert!(is_account_type(
            &TypeNode::Named("UserAccount".to_string()),
            None
        ));
        assert!(is_account_type(
            &TypeNode::Named("CustomAccount".to_string()),
            None
        ));
    }

    #[test]
    fn test_non_account_types() {
        assert!(!is_account_type(&TypeNode::Named("u64".to_string()), None));
        assert!(!is_account_type(
            &TypeNode::Named("String".to_string()),
            None
        ));
        assert!(!is_account_type(&TypeNode::Named("bool".to_string()), None));
        assert!(!is_account_type(
            &TypeNode::Named("MyStruct".to_string()),
            None
        ));
    }

    #[test]
    fn test_account_attributes() {
        assert!(has_account_attributes(&[Attribute {
            name: "mut".to_string(),
            args: vec![]
        }]));
        assert!(has_account_attributes(&[Attribute {
            name: "signer".to_string(),
            args: vec![]
        }]));
        assert!(has_account_attributes(&[Attribute {
            name: "init".to_string(),
            args: vec![]
        }]));
        assert!(has_account_attributes(&[
            Attribute {
                name: "mut".to_string(),
                args: vec![]
            },
            Attribute {
                name: "signer".to_string(),
                args: vec![]
            }
        ]));
        assert!(!has_account_attributes(&[Attribute {
            name: "param".to_string(),
            args: vec![]
        }]));
        assert!(!has_account_attributes(&[]));
    }

    #[test]
    fn test_account_parameter_detection() {
        // Type-based detection
        assert!(is_account_parameter(
            &TypeNode::Named("StateAccount".to_string()),
            &[],
            None
        ));

        // Attribute-based detection
        assert!(is_account_parameter(
            &TypeNode::Named("SomeType".to_string()),
            &[Attribute {
                name: "mut".to_string(),
                args: vec![]
            }],
            None
        ));

        // Init flag detection
        assert!(is_account_parameter(
            &TypeNode::Named("SomeType".to_string()),
            &[Attribute {
                name: "init".to_string(),
                args: vec![]
            }],
            None
        ));

        // Neither type nor attributes indicate account
        assert!(!is_account_parameter(
            &TypeNode::Named("u64".to_string()),
            &[Attribute {
                name: "param".to_string(),
                args: vec![]
            }],
            None
        ));
    }
}
