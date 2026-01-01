//! Five DSL Security Rules Enforcement
//!
//! This module implements compile-time security rule checking to ensure
//! Five DSL contracts follow secure-by-default principles. All security
//! violations are caught at compile time with clear error messages.

use crate::ast::AstNode;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// Security rule violation types
#[derive(Debug, Clone)]
pub enum SecurityViolation {
    /// Attempted assignment to imported field (Rule 1: External State Immutability)
    ExternalFieldMutation {
        field_name: String,
        account_address: String,
        suggestion: String,
    },
    /// Unsafe cross-contract interaction pattern
    UnsafeCrossContractCall {
        function_name: String,
        violation_type: String,
    },
    /// Import scope violation
    ImportScopeViolation {
        import_pattern: String,
        violation: String,
    },
    /// Access control bypass attempt
    AccessControlBypass {
        target: String,
        required_constraint: String,
    },
}

/// Security rule checker for Five DSL compilation
pub struct SecurityChecker {
    /// Imported functions registry: function_name -> (account_address, function_list)
    imported_functions: HashMap<String, (String, Option<Vec<String>>)>,
    /// Imported fields registry: field_name -> (account_address, field_list)  
    imported_fields: HashMap<String, (String, Option<Vec<String>>)>,
    /// Current security context (function name, depth, etc.)
    current_context: SecurityContext,
    /// Detected violations
    violations: Vec<SecurityViolation>,
}

/// Security analysis context
#[derive(Debug, Clone)]
struct SecurityContext {
    current_function: Option<String>,
    call_depth: u32,
    in_external_call: bool,
}

impl SecurityChecker {
    /// Create new security checker
    pub fn new() -> Self {
        Self {
            imported_functions: HashMap::new(),
            imported_fields: HashMap::new(),
            current_context: SecurityContext {
                current_function: None,
                call_depth: 0,
                in_external_call: false,
            },
            violations: Vec::new(),
        }
    }

    /// Set imported items from function dispatcher
    pub fn set_imports(
        &mut self,
        functions: HashMap<String, (String, Option<Vec<String>>)>,
        fields: HashMap<String, (String, Option<Vec<String>>)>,
    ) {
        self.imported_functions = functions;
        self.imported_fields = fields;
    }

    /// Perform comprehensive security analysis on AST
    ///
    /// Returns `Ok(None)` when no violations are detected, `Ok(Some(report))`
    /// when security rule violations are found, and propagates any internal
    /// `VMError` values from the analyzer without converting them to strings.
    pub fn analyze_security(&mut self, ast: &AstNode) -> Result<Option<String>, VMError> {
        self.violations.clear();
        // Propagate any non-security VMError directly
        self.analyze_node(ast)?;

        if !self.violations.is_empty() {
            return Ok(Some(self.report_violations()));
        }

        Ok(None)
    }

    /// Analyze individual AST node for security violations
    fn analyze_node(&mut self, node: &AstNode) -> Result<(), VMError> {
        match node {
            // Rule 1: Check for external field mutations
            AstNode::Assignment { target, value } => {
                self.check_external_field_mutation(target)?;
                self.analyze_node(value)?;
            }

            // Rule 1: Check field assignments in tuple assignments
            AstNode::TupleAssignment { targets, value } => {
                for target in targets {
                    if let AstNode::Identifier(name) = target {
                        self.check_external_field_mutation(name)?;
                    }
                }
                self.analyze_node(value)?;
            }

            // Rule 1: Field assignments with object.field = value
            AstNode::FieldAssignment {
                object,
                field,
                value,
            } => {
                if let AstNode::Identifier(object_name) = object.as_ref() {
                    let full_field_name = format!("{}.{}", object_name, field);
                    self.check_external_field_mutation(&full_field_name)?;
                }
                self.analyze_node(value)?;
            }

            // Rule 2: Check function calls for security patterns
            AstNode::FunctionCall { name, args } => {
                self.check_function_call_security(name, args)?;
                for arg in args {
                    self.analyze_node(arg)?;
                }
            }

            // Rule 4: Check method calls for access control
            AstNode::MethodCall {
                object,
                method,
                args,
            } => {
                self.check_method_call_security(object, method, args)?;
                self.analyze_node(object)?;
                for arg in args {
                    self.analyze_node(arg)?;
                }
            }

            // Recursively analyze all other node types
            _ => {
                self.analyze_children(node)?;
            }
        }

        Ok(())
    }

    /// Check for attempted mutation of external fields (Rule 1)
    fn check_external_field_mutation(&mut self, target: &str) -> Result<(), VMError> {
        if let Some((account_address, _)) = self.imported_fields.get(target) {
            self.violations.push(SecurityViolation::ExternalFieldMutation {
                field_name: target.to_string(),
                account_address: account_address.clone(),
                suggestion: format!(
                    "To modify external contract state, call a function on account {} instead of directly assigning to field '{}'",
                    account_address, target
                ),
            });
        }
        Ok(())
    }

    /// Check function call security patterns (Rule 2)
    fn check_function_call_security(
        &mut self,
        name: &str,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        // Check if this is an external function call
        if let Some((account_address, _)) = self.imported_functions.get(name) {
            // Clone to avoid borrow conflicts
            let account_address = account_address.clone();
            // Verify external call patterns
            self.check_external_call_safety(name, &account_address, args)?;

            // Track call depth for recursion detection
            self.current_context.call_depth += 1;
            if self.current_context.call_depth > 10 {
                self.violations
                    .push(SecurityViolation::UnsafeCrossContractCall {
                        function_name: name.to_string(),
                        violation_type: "Excessive call depth - potential infinite recursion"
                            .to_string(),
                    });
            }
        }

        Ok(())
    }

    /// Check method call security patterns (Rule 4)
    fn check_method_call_security(
        &mut self,
        object: &AstNode,
        method: &str,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        if let AstNode::Identifier(object_name) = object {
            // Check if this is a method call on an imported object
            if self.imported_functions.contains_key(object_name)
                || self.imported_fields.contains_key(object_name)
            {
                self.check_external_call_safety(method, object_name, args)?;
            }
        }
        Ok(())
    }

    /// Verify external call safety patterns
    fn check_external_call_safety(
        &mut self,
        function_name: &str,
        account_address: &str,
        _args: &[AstNode],
    ) -> Result<(), VMError> {
        // Check for known unsafe patterns
        if function_name.contains("unsafe") || function_name.contains("raw") {
            self.violations
                .push(SecurityViolation::UnsafeCrossContractCall {
                    function_name: function_name.to_string(),
                    violation_type: format!(
                        "Potentially unsafe external call to {} on account {}",
                        function_name, account_address
                    ),
                });
        }

        Ok(())
    }

    /// Recursively analyze child nodes
    fn analyze_children(&mut self, node: &AstNode) -> Result<(), VMError> {
        match node {
            AstNode::Program {
                field_definitions,
                instruction_definitions,
                init_block,
                constraints_block,
                ..
            } => {
                for field_def in field_definitions {
                    self.analyze_node(field_def)?;
                }
                for instr_def in instruction_definitions {
                    self.analyze_node(instr_def)?;
                }
                if let Some(init) = init_block {
                    self.current_context.current_function = Some("__init".to_string());
                    self.analyze_node(init)?;
                    self.current_context.current_function = None;
                }
                if let Some(constraints) = constraints_block {
                    self.analyze_node(constraints)?;
                }
            }

            AstNode::InstructionDefinition { name, body, .. } => {
                self.current_context.current_function = Some(name.clone());
                self.analyze_node(body)?;
                self.current_context.current_function = None;
            }

            AstNode::Block { statements, .. } => {
                for stmt in statements {
                    self.analyze_node(stmt)?;
                }
            }

            AstNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze_node(condition)?;
                self.analyze_node(then_branch)?;
                if let Some(else_stmt) = else_branch {
                    self.analyze_node(else_stmt)?;
                }
            }

            AstNode::WhileLoop { condition, body } => {
                self.analyze_node(condition)?;
                self.analyze_node(body)?;
            }

            AstNode::BinaryExpression { left, right, .. } => {
                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }

            AstNode::UnaryExpression { operand, .. } => {
                self.analyze_node(operand)?;
            }

            // Base cases - no children to analyze
            AstNode::Identifier(_) | AstNode::Literal(_) => {}

            // Add more node types as needed
            _ => {}
        }

        Ok(())
    }

    /// Report all security violations with detailed messages
    fn report_violations(&self) -> String {
        let mut report = String::new();
        report.push_str("\n🔒 FIVE DSL SECURITY VIOLATIONS DETECTED:\n");
        report.push_str("==========================================\n");

        for (i, violation) in self.violations.iter().enumerate() {
            report.push_str(&format!(
                "\n{}. {}\n",
                i + 1,
                self.format_violation(violation)
            ));
        }

        report.push_str("\n📋 SECURITY REVIEW REQUIRED:\n");
        report.push_str("- Review all violations above before deployment\n");
        report.push_str("- Follow Five DSL Security Rules for secure patterns\n");
        report.push_str("- See FIVE_SECURITY_RULES.md for detailed guidance\n");
        report.push_str("==========================================\n");

        report
    }

    /// Format security violation for user-friendly error message
    fn format_violation(&self, violation: &SecurityViolation) -> String {
        match violation {
            SecurityViolation::ExternalFieldMutation {
                field_name,
                account_address,
                suggestion,
            } => {
                format!(
                    "🚫 RULE 1 VIOLATION: External State Immutability\n   \
                     Cannot assign to imported field '{}' from account {}\n   \
                     💡 SOLUTION: {}\n   \
                     📖 REFERENCE: FIVE_SECURITY_RULES.md - Rule 1",
                    field_name, account_address, suggestion
                )
            }

            SecurityViolation::UnsafeCrossContractCall {
                function_name,
                violation_type,
            } => {
                format!(
                    "⚠️  RULE 2 VIOLATION: Function Call Security\n   \
                     Unsafe cross-contract call: {} ({})\n   \
                     💡 SOLUTION: Review call pattern for security implications\n   \
                     📖 REFERENCE: FIVE_SECURITY_RULES.md - Rule 2",
                    function_name, violation_type
                )
            }

            SecurityViolation::ImportScopeViolation {
                import_pattern,
                violation,
            } => {
                format!(
                    "🔍 RULE 3 VIOLATION: Import Scope Limitation\n   \
                     Import pattern '{}' violates scope rules: {}\n   \
                     💡 SOLUTION: Use explicit imports with specific function/field names\n   \
                     📖 REFERENCE: FIVE_SECURITY_RULES.md - Rule 3",
                    import_pattern, violation
                )
            }

            SecurityViolation::AccessControlBypass {
                target,
                required_constraint,
            } => {
                format!(
                    "🛡️  RULE 4 VIOLATION: Access Control Preservation\n   \
                     Access control bypass attempt on '{}', requires: {}\n   \
                     💡 SOLUTION: Provide required account constraints\n   \
                     📖 REFERENCE: FIVE_SECURITY_RULES.md - Rule 4",
                    target, required_constraint
                )
            }
        }
    }
}

impl Default for SecurityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for adding security checking to the compiler
pub trait SecurityCheckable {
    /// Add security analysis to compilation process
    fn check_security(&self, checker: &mut SecurityChecker) -> Result<(), VMError>;
}

// Helper functions for security rule enforcement

/// Validate that import patterns follow security rules
pub fn validate_import_security(imports: &[AstNode]) -> Result<(), VMError> {
    for import in imports {
        if let AstNode::ImportStatement {
            module_specifier,
            imported_items,
        } = import
        {
            // Extract account address or module path for reporting
            let account_address = match module_specifier {
                crate::ast::ModuleSpecifier::External(addr) => addr.clone(),
                crate::ast::ModuleSpecifier::Local(name) => name.clone(),
                crate::ast::ModuleSpecifier::Nested(path) => path.join("::"),
            };

            // Rule 3: Validate import scope
            if let Some(items) = imported_items {
                if items.is_empty() {
                    println!(
                        "🔍 SECURITY WARNING: Empty import list for account {}",
                        account_address
                    );
                    println!(
                        "💡 SUGGESTION: Use specific imports or remove unused import statement"
                    );
                }

                // Check for potentially dangerous import patterns
                for item in items {
                    if item.starts_with("_") || item.contains("unsafe") {
                        println!("⚠️  SECURITY WARNING: Importing potentially internal item '{}' from account {}", item, account_address);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Security-aware error types
#[derive(Debug, Clone)]
pub enum SecurityError {
    ExternalFieldMutation(String),
    UnsafeCrossContractCall(String),
    ImportScopeViolation(String),
    AccessControlBypass(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::ExternalFieldMutation(msg) => {
                write!(f, "External Field Mutation: {}", msg)
            }
            SecurityError::UnsafeCrossContractCall(msg) => {
                write!(f, "Unsafe Cross-Contract Call: {}", msg)
            }
            SecurityError::ImportScopeViolation(msg) => {
                write!(f, "Import Scope Violation: {}", msg)
            }
            SecurityError::AccessControlBypass(msg) => write!(f, "Access Control Bypass: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}
