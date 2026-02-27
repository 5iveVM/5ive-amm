// Type helper functions and constants to reduce DRY violations

use crate::ast::TypeNode;

// Constants for built-in type names
pub mod type_names {
    // Numeric types
    pub const U8: &str = "u8";
    pub const U16: &str = "u16";
    pub const U32: &str = "u32";
    pub const U64: &str = "u64";
    pub const U128: &str = "u128";
    pub const I8: &str = "i8";
    pub const I16: &str = "i16";
    pub const I32: &str = "i32";
    pub const I64: &str = "i64";

    // Other primitives
    pub const BOOL: &str = "bool";
    pub const STRING: &str = "string";
    pub const PUBKEY: &str = "pubkey";
    pub const LAMPORTS: &str = "lamports";
    // Account types
    pub const ACCOUNT_LOWER: &str = "account";
    pub const ACCOUNT_UPPER: &str = "Account";

    // Generic types
    pub const OPTION: &str = "Option";
    pub const RESULT: &str = "Result";

    // Special types
    pub const UNKNOWN: &str = "Unknown";
    pub const UNKNOWN_ERROR: &str = "UnknownError";
    pub const UNKNOWN_SUCCESS: &str = "UnknownSuccess";
}

// Constants for built-in properties
pub mod property_names {
    pub const LAMPORTS: &str = "lamports";
    pub const OWNER: &str = "owner";
    pub const KEY: &str = "key";
    pub const DATA: &str = "data";
}

// Constants for built-in functions
pub mod function_names {}

// Constants for special identifiers
pub mod special_identifiers {
    pub const NONE: &str = "None";
}

/// Helper trait for TypeNode to reduce repeated pattern matching
impl TypeNode {
    /// Check if this is a numeric primitive type
    pub fn is_numeric(&self) -> bool {
        match self {
            TypeNode::Primitive(name) => is_numeric_type_name(name),
            _ => false,
        }
    }

    /// Check if this is a boolean type
    pub fn is_bool(&self) -> bool {
        matches!(self, TypeNode::Primitive(name) if name == type_names::BOOL)
    }

    /// Check if this is a string type
    pub fn is_string(&self) -> bool {
        matches!(self, TypeNode::Primitive(name) if name == type_names::STRING)
    }

    /// Check if this is a pubkey type
    pub fn is_pubkey(&self) -> bool {
        matches!(self, TypeNode::Primitive(name) if name == type_names::PUBKEY)
    }

    /// Check if this is an account type (built-in Account or named account types)
    pub fn is_account_type(&self) -> bool {
        match self {
            TypeNode::Account => true,
            TypeNode::Primitive(name) => {
                name == type_names::ACCOUNT_LOWER || name == type_names::ACCOUNT_UPPER
            }
            TypeNode::Named(name) => {
                name == type_names::ACCOUNT_LOWER || name == type_names::ACCOUNT_UPPER
            }
            _ => false,
        }
    }

    /// Check if this is an Option<T> type
    pub fn is_option(&self) -> bool {
        matches!(self, TypeNode::Generic { base, .. } if base == type_names::OPTION)
    }

    /// Check if this is a Result<T, E> type
    pub fn is_result(&self) -> bool {
        matches!(self, TypeNode::Generic { base, .. } if base == type_names::RESULT)
    }

    /// Check if this is an unknown placeholder type
    pub fn is_unknown_placeholder(&self) -> bool {
        matches!(
            self,
            TypeNode::Named(name) if name == type_names::UNKNOWN
                || name == type_names::UNKNOWN_ERROR
                || name == type_names::UNKNOWN_SUCCESS
        )
    }

    /// Extract the inner type from Option<T>, if applicable
    pub fn option_inner(&self) -> Option<&TypeNode> {
        match self {
            TypeNode::Generic { base, args } if base == type_names::OPTION && !args.is_empty() => {
                Some(&args[0])
            }
            _ => None,
        }
    }

    /// Extract the success type from Result<T, E>, if applicable
    pub fn result_ok_type(&self) -> Option<&TypeNode> {
        match self {
            TypeNode::Generic { base, args } if base == type_names::RESULT && !args.is_empty() => {
                Some(&args[0])
            }
            _ => None,
        }
    }

    /// Create a primitive type
    pub fn primitive(name: &str) -> Self {
        TypeNode::Primitive(name.to_string())
    }

    /// Create an Option<T> type
    pub fn option(inner: TypeNode) -> Self {
        TypeNode::Generic {
            base: type_names::OPTION.to_string(),
            args: vec![inner],
        }
    }

    /// Create a Result<T, E> type
    pub fn result(ok: TypeNode, err: TypeNode) -> Self {
        TypeNode::Generic {
            base: type_names::RESULT.to_string(),
            args: vec![ok, err],
        }
    }
}

/// Check if a type name string represents a numeric type
pub fn is_numeric_type_name(name: &str) -> bool {
    matches!(
        name,
        type_names::U8
            | type_names::U16
            | type_names::U32
            | type_names::U64
            | type_names::U128
            | type_names::I8
            | type_names::I16
            | type_names::I32
            | type_names::I64
            | type_names::LAMPORTS
    )
}

/// Check if a string is a primitive type name (numeric, bool, string, pubkey, etc.)
pub fn is_primitive_type_name(name: &str) -> bool {
    matches!(
        name,
        type_names::U8
            | type_names::U16
            | type_names::U32
            | type_names::U64
            | type_names::U128
            | type_names::I8
            | type_names::I16
            | type_names::I32
            | type_names::I64
            | type_names::LAMPORTS
            | type_names::BOOL
            | type_names::STRING
            | type_names::PUBKEY
            | type_names::ACCOUNT_LOWER
            | type_names::ACCOUNT_UPPER
    )
}

/// Check if a string is a built-in account property
pub fn is_builtin_account_property(name: &str) -> bool {
    matches!(
        name,
        property_names::LAMPORTS
            | property_names::OWNER
            | property_names::KEY
            | property_names::DATA
    )
}

/// Check if an identifier is a special built-in identifier

/// Get metadata about a numeric type (signed, bits)
pub fn numeric_type_meta(name: &str) -> Option<(bool /*signed*/, u32 /*bits*/)> {
    Some(match name {
        type_names::U8 => (false, 8),
        type_names::U16 => (false, 16),
        type_names::U32 => (false, 32),
        type_names::U64 => (false, 64),
        type_names::LAMPORTS => (false, 64),
        type_names::U128 => (false, 128),
        type_names::I8 => (true, 8),
        type_names::I16 => (true, 16),
        type_names::I32 => (true, 32),
        type_names::I64 => (true, 64),
        _ => return None,
    })
}

/// Macro to check if a value matches any of the given patterns
#[macro_export]
macro_rules! matches_any {
    // `pat_param` lets us accept multiple `|` separators in pattern fragments.
    ($expr:expr, $($pattern:pat_param)|+) => {
        matches!($expr, $($pattern)|+)
    };
}

/// Macro to create a primitive TypeNode
#[macro_export]
macro_rules! primitive_type {
    ($name:expr) => {
        TypeNode::Primitive($name.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numeric() {
        assert!(TypeNode::primitive(type_names::U8).is_numeric());
        assert!(TypeNode::primitive(type_names::U64).is_numeric());
        assert!(TypeNode::primitive(type_names::I32).is_numeric());
        assert!(!TypeNode::primitive(type_names::BOOL).is_numeric());
        assert!(!TypeNode::primitive(type_names::STRING).is_numeric());
        assert!(!TypeNode::Account.is_numeric());
    }

    #[test]
    fn test_is_bool() {
        assert!(TypeNode::primitive(type_names::BOOL).is_bool());
        assert!(!TypeNode::primitive(type_names::U64).is_bool());
        assert!(!TypeNode::primitive(type_names::STRING).is_bool());
    }

    #[test]
    fn test_is_string() {
        assert!(TypeNode::primitive(type_names::STRING).is_string());
        assert!(!TypeNode::primitive(type_names::BOOL).is_string());
        assert!(!TypeNode::primitive(type_names::U64).is_string());
    }

    #[test]
    fn test_is_pubkey() {
        assert!(TypeNode::primitive(type_names::PUBKEY).is_pubkey());
        assert!(!TypeNode::primitive(type_names::STRING).is_pubkey());
        assert!(!TypeNode::Account.is_pubkey());
    }

    #[test]
    fn test_is_account_type() {
        assert!(TypeNode::Account.is_account_type());
        assert!(TypeNode::Named(type_names::ACCOUNT_LOWER.to_string()).is_account_type());
        assert!(TypeNode::Named(type_names::ACCOUNT_UPPER.to_string()).is_account_type());
        assert!(!TypeNode::primitive(type_names::STRING).is_account_type());
        assert!(!TypeNode::Named("CustomType".to_string()).is_account_type());
    }

    #[test]
    fn test_is_option() {
        let opt = TypeNode::option(TypeNode::primitive(type_names::U64));
        assert!(opt.is_option());
        assert!(!TypeNode::primitive(type_names::U64).is_option());

        let result = TypeNode::result(
            TypeNode::primitive(type_names::U64),
            TypeNode::primitive(type_names::STRING),
        );
        assert!(!result.is_option());
    }

    #[test]
    fn test_is_result() {
        let result = TypeNode::result(
            TypeNode::primitive(type_names::U64),
            TypeNode::primitive(type_names::STRING),
        );
        assert!(result.is_result());
        assert!(!TypeNode::primitive(type_names::U64).is_result());

        let opt = TypeNode::option(TypeNode::primitive(type_names::U64));
        assert!(!opt.is_result());
    }

    #[test]
    fn test_is_unknown_placeholder() {
        assert!(TypeNode::Named(type_names::UNKNOWN.to_string()).is_unknown_placeholder());
        assert!(TypeNode::Named(type_names::UNKNOWN_ERROR.to_string()).is_unknown_placeholder());
        assert!(TypeNode::Named(type_names::UNKNOWN_SUCCESS.to_string()).is_unknown_placeholder());
        assert!(!TypeNode::Named("CustomType".to_string()).is_unknown_placeholder());
        assert!(!TypeNode::primitive(type_names::U64).is_unknown_placeholder());
    }

    #[test]
    fn test_option_inner() {
        let opt = TypeNode::option(TypeNode::primitive(type_names::U64));
        let inner = opt.option_inner();
        assert!(inner.is_some());
        assert!(inner.unwrap().is_numeric());

        let not_opt = TypeNode::primitive(type_names::U64);
        assert!(not_opt.option_inner().is_none());
    }

    #[test]
    fn test_result_ok_type() {
        let result = TypeNode::result(
            TypeNode::primitive(type_names::U64),
            TypeNode::primitive(type_names::STRING),
        );
        let ok_type = result.result_ok_type();
        assert!(ok_type.is_some());
        assert!(ok_type.unwrap().is_numeric());

        let not_result = TypeNode::primitive(type_names::U64);
        assert!(not_result.result_ok_type().is_none());
    }

    #[test]
    fn test_factory_methods() {
        let prim = TypeNode::primitive(type_names::U64);
        assert!(matches!(prim, TypeNode::Primitive(ref name) if name == type_names::U64));

        let opt = TypeNode::option(TypeNode::primitive(type_names::U64));
        assert!(opt.is_option());

        let res = TypeNode::result(
            TypeNode::primitive(type_names::U64),
            TypeNode::primitive(type_names::STRING),
        );
        assert!(res.is_result());
    }

    #[test]
    fn test_is_numeric_type_name() {
        assert!(is_numeric_type_name(type_names::U8));
        assert!(is_numeric_type_name(type_names::U64));
        assert!(is_numeric_type_name(type_names::I32));
        assert!(!is_numeric_type_name(type_names::BOOL));
        assert!(!is_numeric_type_name(type_names::STRING));
        assert!(!is_numeric_type_name("unknown"));
    }

    #[test]
    fn test_is_builtin_account_property() {
        assert!(is_builtin_account_property(property_names::LAMPORTS));
        assert!(is_builtin_account_property(property_names::OWNER));
        assert!(is_builtin_account_property(property_names::KEY));
        assert!(is_builtin_account_property(property_names::DATA));
        assert!(!is_builtin_account_property("custom_field"));
    }



    #[test]
    fn test_numeric_type_meta() {
        assert_eq!(numeric_type_meta(type_names::U8), Some((false, 8)));
        assert_eq!(numeric_type_meta(type_names::U64), Some((false, 64)));
        assert_eq!(numeric_type_meta(type_names::I32), Some((true, 32)));
        assert_eq!(numeric_type_meta(type_names::BOOL), None);
    }
}
