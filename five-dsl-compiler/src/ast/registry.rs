//! Node registry for Five DSL AST metadata
//!
//! This module provides a runtime registry of all AST node definitions loaded from
//! the single-source-of-truth metadata file (`node_metadata.toml`).
//!
//! The registry enables:
//! - Type-safe node introspection
//! - Automatic code generation tools
//! - Validation of node definitions
//! - Query API for node metadata

use std::collections::HashMap;
use std::sync::LazyLock;

/// Node metadata loaded from TOML
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub name: String,
    pub category: String,
    pub doc: String,
    pub fields: HashMap<String, FieldMetadata>,
    pub grammar: Option<GrammarMetadata>,
    pub precedence: Option<i32>,
    pub associativity: Option<String>,
}

/// Field metadata for a node
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    pub name: String,
    pub field_type: String,
    pub doc: String,
}

/// Grammar rule metadata
#[derive(Debug, Clone)]
pub struct GrammarMetadata {
    pub rule_name: String,
    pub rule: String,
}

/// Auxiliary type metadata
#[derive(Debug, Clone)]
pub struct AuxiliaryTypeMetadata {
    pub name: String,
    pub fields: HashMap<String, String>,
    pub doc: String,
}

/// Type node metadata
#[derive(Debug, Clone)]
pub struct TypeNodeMetadata {
    pub name: String,
    pub fields: HashMap<String, String>,
    pub doc: String,
}

/// Main registry for all node metadata
pub struct NodeRegistry {
    /// All AST nodes indexed by name
    pub nodes: HashMap<String, NodeMetadata>,

    /// All auxiliary types
    pub auxiliary_types: HashMap<String, AuxiliaryTypeMetadata>,

    /// All type nodes
    pub type_nodes: HashMap<String, TypeNodeMetadata>,

    /// Node categories for querying
    pub categories: HashMap<String, Vec<String>>,

    /// Metadata about the registry itself
    pub metadata: RegistryMetadata,
}

/// Metadata about the registry
#[derive(Debug, Clone)]
pub struct RegistryMetadata {
    pub version: String,
    pub total_ast_nodes: usize,
    pub total_type_nodes: usize,
    pub total_auxiliary_types: usize,
    pub description: String,
}

impl NodeRegistry {
    /// Create and load the registry from embedded TOML
    pub fn load() -> Result<Self, RegistryError> {
        let toml_content = include_str!("../../node_metadata.toml");
        Self::parse(toml_content)
    }

    /// Parse TOML metadata into a registry
    fn parse(toml_str: &str) -> Result<Self, RegistryError> {
        let value: toml::Value = toml::from_str(toml_str)
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        let mut registry = Self {
            nodes: HashMap::new(),
            auxiliary_types: HashMap::new(),
            type_nodes: HashMap::new(),
            categories: HashMap::new(),
            metadata: RegistryMetadata {
                version: String::new(),
                total_ast_nodes: 0,
                total_type_nodes: 0,
                total_auxiliary_types: 0,
                description: String::new(),
            },
        };

        // Load metadata
        if let Some(meta) = value.get("metadata").and_then(|m| m.as_table()) {
            registry.metadata.version = meta
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            registry.metadata.total_ast_nodes = meta
                .get("total_ast_nodes")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as usize;
            registry.metadata.total_type_nodes = meta
                .get("total_type_nodes")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as usize;
            registry.metadata.total_auxiliary_types = meta
                .get("total_auxiliary_types")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as usize;
            registry.metadata.description = meta
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }

        // Load auxiliary types
        if let Some(aux_array) = value.get("auxiliary_types").and_then(|a| a.as_array()) {
            for aux_item in aux_array {
                if let Some(table) = aux_item.as_table() {
                    let name = table
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or(RegistryError::MissingField("auxiliary type name".to_string()))?
                        .to_string();

                    let mut fields = HashMap::new();
                    if let Some(field_table) = table.get("fields").and_then(|f| f.as_table()) {
                        for (field_name, field_value) in field_table {
                            let field_type = field_value
                                .as_str()
                                .or_else(|| {
                                    field_value.get("type").and_then(|t| t.as_str())
                                })
                                .ok_or_else(|| {
                                    RegistryError::InvalidFormat(format!(
                                        "Invalid field type for {}",
                                        field_name
                                    ))
                                })?
                                .to_string();
                            fields.insert(field_name.clone(), field_type);
                        }
                    }

                    let doc = table
                        .get("doc")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();

                    registry.auxiliary_types.insert(
                        name.clone(),
                        AuxiliaryTypeMetadata {
                            name,
                            fields,
                            doc,
                        },
                    );
                }
            }
        }

        // Load type nodes
        if let Some(type_array) = value.get("type_nodes").and_then(|a| a.as_array()) {
            for type_item in type_array {
                if let Some(table) = type_item.as_table() {
                    let name = table
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or(RegistryError::MissingField("type node name".to_string()))?
                        .to_string();

                    let mut fields = HashMap::new();
                    if let Some(field_table) = table.get("fields").and_then(|f| f.as_table()) {
                        for (field_name, field_value) in field_table {
                            let field_type = field_value
                                .get("type")
                                .and_then(|t| t.as_str())
                                .ok_or_else(|| {
                                    RegistryError::InvalidFormat(format!(
                                        "Invalid field type for {}",
                                        field_name
                                    ))
                                })?
                                .to_string();
                            fields.insert(field_name.clone(), field_type);
                        }
                    } else if let Some(single_field) = table.get("field").and_then(|f| f.as_str())
                    {
                        // Handle single field shorthand
                        fields.insert("value".to_string(), single_field.to_string());
                    }

                    let doc = table
                        .get("doc")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();

                    registry.type_nodes.insert(
                        name.clone(),
                        TypeNodeMetadata {
                            name,
                            fields,
                            doc,
                        },
                    );
                }
            }
        }

        // Load AST nodes
        if let Some(nodes_array) = value.get("nodes").and_then(|n| n.as_array()) {
            for node_item in nodes_array {
                if let Some(table) = node_item.as_table() {
                    let name = table
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or(RegistryError::MissingField("node name".to_string()))?
                        .to_string();

                    let category = table
                        .get("category")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| {
                            RegistryError::MissingField(format!("category for node {}", name))
                        })?
                        .to_string();

                    let doc = table
                        .get("doc")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Parse fields
                    let mut fields = HashMap::new();
                    if let Some(field_table) = table.get("fields").and_then(|f| f.as_table()) {
                        for (field_name, field_value) in field_table {
                            if let Some(field_obj) = field_value.as_table() {
                                let field_type = field_obj
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .ok_or_else(|| {
                                        RegistryError::InvalidFormat(format!(
                                            "Missing type for field {}",
                                            field_name
                                        ))
                                    })?
                                    .to_string();

                                let field_doc = field_obj
                                    .get("doc")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                fields.insert(
                                    field_name.clone(),
                                    FieldMetadata {
                                        name: field_name.clone(),
                                        field_type,
                                        doc: field_doc,
                                    },
                                );
                            }
                        }
                    }

                    // Parse grammar metadata - look for [nodes.grammar] section in same table
                    let grammar = table
                        .get("grammar")
                        .and_then(|g| g.as_table())
                        .map(|gg| {
                            GrammarMetadata {
                                rule_name: gg
                                    .get("rule_name")
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                rule: gg
                                    .get("rule")
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            }
                        });

                    let precedence = table.get("precedence").and_then(|p| p.as_integer()).map(|p| p as i32);
                    let associativity = table
                        .get("associativity")
                        .and_then(|a| a.as_str())
                        .map(|a| a.to_string());

                    let node = NodeMetadata {
                        name: name.clone(),
                        category: category.clone(),
                        doc,
                        fields,
                        grammar,
                        precedence,
                        associativity,
                    };

                    registry.nodes.insert(name.clone(), node);
                    registry
                        .categories
                        .entry(category)
                        .or_default()
                        .push(name);
                }
            }
        }

        registry.validate()?;
        Ok(registry)
    }

    /// Validate registry consistency
    fn validate(&self) -> Result<(), RegistryError> {
        // Check all nodes have required fields
        if self.nodes.is_empty() {
            return Err(RegistryError::ValidationError(
                "No nodes loaded from metadata".to_string(),
            ));
        }

        // Verify expected node count
        if self.nodes.len() != self.metadata.total_ast_nodes {
            return Err(RegistryError::ValidationError(format!(
                "Expected {} nodes but loaded {}",
                self.metadata.total_ast_nodes,
                self.nodes.len()
            )));
        }

        Ok(())
    }

    /// Get a node by name
    pub fn get_node(&self, name: &str) -> Option<&NodeMetadata> {
        self.nodes.get(name)
    }

    /// Get all nodes in a category
    pub fn get_by_category(&self, category: &str) -> Vec<&NodeMetadata> {
        self.categories
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| self.nodes.get(n))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all node names
    pub fn get_all_node_names(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    /// Get all categories
    pub fn get_all_categories(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    /// Get auxiliary type by name
    pub fn get_auxiliary_type(&self, name: &str) -> Option<&AuxiliaryTypeMetadata> {
        self.auxiliary_types.get(name)
    }

    /// Get type node by name
    pub fn get_type_node(&self, name: &str) -> Option<&TypeNodeMetadata> {
        self.type_nodes.get(name)
    }
}

/// Global node registry singleton
pub static NODE_REGISTRY: LazyLock<NodeRegistry> = LazyLock::new(|| {
    NodeRegistry::load().expect("Failed to load node registry from node_metadata.toml")
});

/// Registry error types
#[derive(Debug, Clone)]
pub enum RegistryError {
    ParseError(String),
    MissingField(String),
    InvalidFormat(String),
    ValidationError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::MissingField(field) => write!(f, "Missing field: {}", field),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_loads() {
        let registry = &*NODE_REGISTRY;
        assert!(!registry.nodes.is_empty());
    }

    #[test]
    fn test_registry_validates() {
        let registry = &*NODE_REGISTRY;
        assert_eq!(registry.nodes.len(), registry.metadata.total_ast_nodes);
    }

    #[test]
    fn test_get_by_category() {
        let registry = &*NODE_REGISTRY;
        let statements = registry.get_by_category("statement");
        assert!(!statements.is_empty());
    }
}
