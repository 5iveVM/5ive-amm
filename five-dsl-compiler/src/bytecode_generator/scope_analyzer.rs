// Scope analysis for local variable optimization.

use crate::ast::AstNode;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// Variable scope information for local variable optimization
#[derive(Debug, Clone)]
pub struct VariableScope {
    pub name: String,
    pub var_type: String,
    pub scope_level: usize,
    pub first_use: usize,
    pub last_use: usize,
    pub assigned_slot: Option<usize>,
    pub usage_count: usize,
    pub is_parameter: bool,
}

/// Scope analysis results for a function
#[derive(Debug, Clone)]
pub struct FunctionScopeAnalysis {
    pub function_name: String,
    pub variables: Vec<VariableScope>,
    pub max_scope_level: usize,
    pub total_local_slots: usize,
    pub scope_stack: Vec<usize>, // Track scope nesting
}

/// Enhanced Scope Analyzer for local variable optimization
pub struct ScopeAnalyzer {
    /// Current function being analyzed
    pub current_function: Option<String>,

    /// Complete scope analyses for all functions
    pub scope_analyses: HashMap<String, FunctionScopeAnalysis>,

    /// Current scope nesting level
    pub current_scope_level: usize,

    /// Instruction counter for tracking variable lifetimes
    pub instruction_counter: usize,

    /// Variable tracking within current scope
    current_variables: HashMap<String, VariableScope>,

    /// Scope stack for tracking nested scopes
    scope_stack: Vec<usize>,
}

impl ScopeAnalyzer {
    /// Create a new scope analyzer
    pub fn new() -> Self {
        Self {
            current_function: None,
            scope_analyses: HashMap::new(),
            current_scope_level: 0,
            instruction_counter: 0,
            current_variables: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }

    /// Analyze scope structure for an entire AST
    pub fn analyze_program(&mut self, ast: &AstNode) -> Result<(), VMError> {
        match ast {
            AstNode::Program {
                instruction_definitions,
                init_block,
                ..
            } => {
                // Analyze init block if present
                if let Some(init) = init_block {
                    self.begin_function("__init".to_string())?;
                    self.analyze_block(init)?;
                    self.end_function()?;
                }

                // Analyze all instruction definitions
                for instruction_def in instruction_definitions {
                    self.analyze_instruction_definition(instruction_def)?;
                }

                Ok(())
            }
            _ => Err(VMError::InvalidScript),
        }
    }

    /// Analyze a function/instruction definition
    pub fn analyze_instruction_definition(
        &mut self,
        instruction_def: &AstNode,
    ) -> Result<(), VMError> {
        match instruction_def {
            AstNode::InstructionDefinition {
                name,
                parameters,
                body,
                ..
            } => {
                self.begin_function(name.clone())?;

                // Add parameters as variables in the function scope
                for param in parameters {
                    self.declare_variable(
                        &param.name,
                        &self.type_node_to_string(&param.param_type),
                        true, // is_parameter
                    )?;
                }

                // Analyze function body
                self.analyze_node(body)?;

                self.end_function()?;
                Ok(())
            }
            _ => Ok(()), // Skip non-instruction definitions
        }
    }

    /// Begin analysis of a new function
    pub fn begin_function(&mut self, function_name: String) -> Result<(), VMError> {
        self.current_function = Some(function_name.clone());
        self.current_scope_level = 0;
        self.instruction_counter = 0;
        self.current_variables.clear();
        self.scope_stack.clear();

        // Initialize function analysis
        self.scope_analyses.insert(
            function_name.clone(),
            FunctionScopeAnalysis {
                function_name,
                variables: Vec::new(),
                max_scope_level: 0,
                total_local_slots: 0,
                scope_stack: Vec::new(),
            },
        );

        Ok(())
    }

    /// End analysis of the current function
    pub fn end_function(&mut self) -> Result<(), VMError> {
        if let Some(function_name) = &self.current_function {
            // Finalize all variables in the current function
            for var_scope in self.current_variables.values() {
                if let Some(analysis) = self.scope_analyses.get_mut(function_name) {
                    analysis.variables.push(var_scope.clone());
                }
            }

            // Update final statistics
            if let Some(analysis) = self.scope_analyses.get_mut(function_name) {
                // Note: max_scope_level is already tracked incrementally in enter_scope()
                // Do not overwrite it here with current_scope_level (which is 0 after all scopes exit)
                analysis.total_local_slots = analysis.variables.len();
                // scope_stack is tracked during analysis, no need to capture empty final state
            }
        }

        self.current_function = None;
        Ok(())
    }

    /// Enter a new scope level
    pub fn enter_scope(&mut self) -> Result<(), VMError> {
        self.current_scope_level += 1;
        self.scope_stack.push(self.instruction_counter);

        // Update max scope level
        if let Some(function_name) = &self.current_function {
            if let Some(analysis) = self.scope_analyses.get_mut(function_name) {
                analysis.max_scope_level = analysis.max_scope_level.max(self.current_scope_level);
            }
        }

        Ok(())
    }

    /// Exit the current scope level
    pub fn exit_scope(&mut self) -> Result<(), VMError> {
        if self.current_scope_level > 0 {
            self.current_scope_level -= 1;
            self.scope_stack.pop();

            // Mark variables that go out of scope
            for var_scope in self.current_variables.values_mut() {
                if var_scope.scope_level > self.current_scope_level {
                    var_scope.last_use = self.instruction_counter;
                }
            }
        }

        Ok(())
    }

    /// Declare a new variable in the current scope
    pub fn declare_variable(
        &mut self,
        name: &str,
        var_type: &str,
        is_parameter: bool,
    ) -> Result<(), VMError> {
        let var_scope = VariableScope {
            name: name.to_string(),
            var_type: var_type.to_string(),
            scope_level: self.current_scope_level,
            first_use: self.instruction_counter,
            last_use: self.instruction_counter,
            assigned_slot: None,
            usage_count: 0,
            is_parameter,
        };

        self.current_variables.insert(name.to_string(), var_scope);
        self.instruction_counter += 1;

        Ok(())
    }

    /// Record variable usage (read or write)
    pub fn use_variable(&mut self, name: &str) -> Result<(), VMError> {
        if let Some(var_scope) = self.current_variables.get_mut(name) {
            var_scope.last_use = self.instruction_counter;
            var_scope.usage_count += 1;
        } else {
            // Variable not found in current scope - could be parameter or global
            // Declare unknown type.
            // Variable not found in current scope - could be parameter or global
            // Declare unknown type (assume local).
            self.declare_variable(name, "unknown", false)?;
        }

        self.instruction_counter += 1;
        Ok(())
    }

    /// Analyze function scope for optimization
    pub fn analyze_function_scope(
        &mut self,
        function_name: &str,
        body: &AstNode,
    ) -> Result<(), VMError> {
        self.current_function = Some(function_name.to_string());
        self.current_scope_level = 0;
        self.instruction_counter = 0;
        self.current_variables.clear();

        // Analyze the function body
        self.analyze_node(body)?;

        // Store the complete analysis
        let analysis = FunctionScopeAnalysis {
            function_name: function_name.to_string(),
            variables: self.current_variables.values().cloned().collect(),
            max_scope_level: self.current_scope_level,
            total_local_slots: self.current_variables.len(),
            scope_stack: self.scope_stack.clone(),
        };

        self.scope_analyses
            .insert(function_name.to_string(), analysis);
        self.current_function = None;

        Ok(())
    }

    /// Analyze a generic AST node for scope information
    pub fn analyze_node(&mut self, node: &AstNode) -> Result<(), VMError> {
        match node {
            AstNode::Block { .. } => {
                self.analyze_block(node)?;
            }
            AstNode::LetStatement { name, value, .. } => {
                // First analyze the value expression
                self.analyze_node(value)?;
                // Then declare the variable
                // Then declare the variable
                self.declare_variable(name, "inferred", false)?;
            }
            AstNode::Assignment { target, value } => {
                // Analyze the value expression first
                self.analyze_node(value)?;
                // Then record usage of the target variable
                self.use_variable(target)?;
            }
            AstNode::Identifier(name) => {
                self.use_variable(name)?;
            }
            AstNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                // Analyze condition
                self.analyze_node(condition)?;

                // Analyze then branch in new scope
                self.enter_scope()?;
                self.analyze_node(then_branch)?;
                self.exit_scope()?;

                // Analyze else branch in new scope if present
                if let Some(else_node) = else_branch {
                    self.enter_scope()?;
                    self.analyze_node(else_node)?;
                    self.exit_scope()?;
                }
            }
            AstNode::MatchExpression { expression, arms } => {
                // Analyze the match expression
                self.analyze_node(expression)?;

                // Analyze each arm in its own scope
                for arm in arms {
                    self.enter_scope()?;
                    self.analyze_node(&arm.pattern)?;
                    self.analyze_node(&arm.body)?;
                    self.exit_scope()?;
                }
            }
            AstNode::BinaryExpression { left, right, .. } => {
                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            AstNode::MethodCall { object, args, .. } => {
                self.analyze_node(object)?;
                for arg in args {
                    self.analyze_node(arg)?;
                }
            }
            AstNode::ReturnStatement { value } => {
                if let Some(val) = value {
                    self.analyze_node(val)?;
                }
                self.instruction_counter += 1;
            }
            _ => {
                // For other node types, just increment counter
                self.instruction_counter += 1;
            }
        }

        Ok(())
    }

    /// Analyze a block of statements
    pub fn analyze_block(&mut self, block: &AstNode) -> Result<(), VMError> {
        match block {
            AstNode::Block { statements, .. } => {
                self.enter_scope()?;

                for statement in statements {
                    self.analyze_node(statement)?;
                }

                self.exit_scope()?;
            }
            _ => {
                // Not a block, analyze as single node
                self.analyze_node(block)?;
            }
        }

        Ok(())
    }

    /// Get scope analysis for a specific function
    pub fn get_function_analysis(&self, function_name: &str) -> Option<&FunctionScopeAnalysis> {
        self.scope_analyses.get(function_name)
    }

    /// Get all function analyses
    pub fn get_all_analyses(&self) -> &HashMap<String, FunctionScopeAnalysis> {
        &self.scope_analyses
    }

    /// Optimize local variable slot allocation based on scope analysis
    pub fn optimize_local_slots(
        &mut self,
        function_name: &str,
    ) -> Result<Vec<(String, usize)>, VMError> {
        let mut allocations = Vec::new();

        if let Some(analysis) = self.scope_analyses.get_mut(function_name) {
            // Sort variables by first use to optimize allocation (Linear Scan Allocation)
            // We must preserve lifetime order for the allocation reusing logic to work correctly
            analysis.variables.sort_by_key(|v| v.first_use);

            let mut available_slots = Vec::new();
            let mut next_slot = 0;
            let variables_len = analysis.variables.len();

            // Map to track usage weight for each allocated slot
            // slot_index -> total_usage_count
            let mut slot_weights: HashMap<usize, usize> = HashMap::new();

            for variable in &mut analysis.variables {
                // Skip parameters - they don't consume local slots
                if variable.is_parameter {
                    continue;
                }

                // Try to reuse a slot from a variable that's no longer needed
                let slot = if let Some(reuse_slot) = available_slots.pop() {
                    reuse_slot
                } else {
                    let slot = next_slot;
                    next_slot += 1;
                    slot
                };

                variable.assigned_slot = Some(slot);
                // Accumulate usage count for this slot
                *slot_weights.entry(slot).or_insert(0) += variable.usage_count;

                // Note: The logic below for freeing slots is simplified and imperfect.
                // In a full implementation, we would check implementation-specific liveness intervals.
                // However, preserving existing behavior for now.
                if variable.last_use < variables_len - 1 {
                    available_slots.push(slot);
                }
            }

            // OPTIMIZATION: Remap slots based on frequency
            // Create a mapping from old_slot -> new_slot where new slots 0,1,2,3
            // are assigned to the most heavily used old slots.

            // 1. Convert weights to a vector of (slot, weight)
            let mut weighted_slots: Vec<(usize, usize)> = slot_weights.into_iter().collect();

            // 2. Sort by weight descending (most used first), then by slot index (stability)
            weighted_slots.sort_by(|a, b| {
                b.1.cmp(&a.1) // Descending by weight
                    .then(a.0.cmp(&b.0)) // Ascending by slot index (stable tie-break)
            });

            // 3. Create mapping: weighted_slots[i].original_slot -> i (new_slot)
            let mut slot_map: HashMap<usize, usize> = HashMap::new();
            for (new_index, (old_slot, _)) in weighted_slots.iter().enumerate() {
                slot_map.insert(*old_slot, new_index);
            }

            // 4. Update all variables with new slot mappings
            allocations.clear(); // Clear any partial work if it existed
            for variable in &mut analysis.variables {
                if let Some(old_slot) = variable.assigned_slot {
                    if let Some(new_slot) = slot_map.get(&old_slot) {
                        variable.assigned_slot = Some(*new_slot);
                        allocations.push((variable.name.clone(), *new_slot));
                    }
                }
            }

            // Update total local slots needed to the peak count
            analysis.total_local_slots = next_slot;
        }

        Ok(allocations)
    }

    /// Generate optimization report
    pub fn generate_optimization_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Scope Analysis Report\n");
        report.push_str("====================\n\n");

        for (function_name, analysis) in &self.scope_analyses {
            report.push_str(&format!("Function: {}\n", function_name));
            report.push_str(&format!("  Variables: {}\n", analysis.variables.len()));
            report.push_str(&format!(
                "  Max scope level: {}\n",
                analysis.max_scope_level
            ));
            report.push_str(&format!(
                "  Local slots needed: {}\n",
                analysis.total_local_slots
            ));

            for variable in &analysis.variables {
                report.push_str(&format!(
                    "  - {} ({}): scope {}, used {}-{}, slot {:?}\n",
                    variable.name,
                    variable.var_type,
                    variable.scope_level,
                    variable.first_use,
                    variable.last_use,
                    variable.assigned_slot
                ));
            }

            report.push('\n');
        }

        report
    }

    /// Helper: Convert TypeNode to string
    fn type_node_to_string(&self, type_node: &crate::ast::TypeNode) -> String {
        use crate::ast::TypeNode;

        match type_node {
            TypeNode::Primitive(name) => name.clone(),
            TypeNode::Generic { base, .. } => base.clone(),
            _ => "unknown".to_string(),
        }
    }
}

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Perform scope analysis on the AST
    pub fn analyze_scopes(&mut self, ast: &AstNode) -> Result<ScopeAnalyzer, VMError> {
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze_program(ast)?;
        Ok(analyzer)
    }

    /// Generate optimized local variable allocation
    pub fn optimize_local_variables(
        &mut self,
        ast: &AstNode,
    ) -> Result<HashMap<String, Vec<(String, usize)>>, VMError> {
        let mut analyzer = self.analyze_scopes(ast)?;
        let mut all_allocations = HashMap::new();

        for function_name in analyzer.scope_analyses.keys().cloned().collect::<Vec<_>>() {
            let allocations = analyzer.optimize_local_slots(&function_name)?;
            all_allocations.insert(function_name, allocations);
        }

        Ok(all_allocations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::ast::{AstNode, InstructionParameter, TypeNode};
    // use five_protocol::Value;

    #[test]
    fn test_basic_scope_analysis() {
        let mut analyzer = ScopeAnalyzer::new();

        // Test function analysis
        analyzer
            .begin_function("test_function".to_string())
            .unwrap();
        analyzer.declare_variable("x", "u64", false).unwrap();
        analyzer.use_variable("x").unwrap();
        analyzer.end_function().unwrap();

        let analysis = analyzer.get_function_analysis("test_function").unwrap();
        assert_eq!(analysis.variables.len(), 1);
        assert_eq!(analysis.variables[0].name, "x");
        assert_eq!(analysis.variables[0].var_type, "u64");
    }

    #[test]
    fn test_nested_scope_analysis() {
        let mut analyzer = ScopeAnalyzer::new();

        analyzer.begin_function("nested_test".to_string()).unwrap();
        analyzer.declare_variable("outer", "u64", false).unwrap();

        analyzer.enter_scope().unwrap();
        analyzer.declare_variable("inner", "bool", false).unwrap();
        analyzer.use_variable("outer").unwrap(); // Use outer variable in inner scope
        analyzer.exit_scope().unwrap();

        analyzer.end_function().unwrap();

        let analysis = analyzer.get_function_analysis("nested_test").unwrap();
        assert_eq!(analysis.max_scope_level, 1);
        assert_eq!(analysis.variables.len(), 2);
    }

    #[test]
    fn test_local_slots_optimization() {
        let mut analyzer = ScopeAnalyzer::new();

        analyzer.begin_function("alloc_test".to_string()).unwrap();
        analyzer.declare_variable("a", "u64", false).unwrap();
        analyzer.declare_variable("b", "u64", false).unwrap();
        analyzer.declare_variable("c", "u64", false).unwrap();
        analyzer.end_function().unwrap();

        let allocations = analyzer.optimize_local_slots("alloc_test").unwrap();

        assert_eq!(allocations.len(), 3);
        // Variables should be allocated to slots
        assert!(allocations.iter().any(|(name, _)| name == "a"));
        assert!(allocations.iter().any(|(name, _)| name == "b"));
        assert!(allocations.iter().any(|(name, _)| name == "c"));
    }
}
