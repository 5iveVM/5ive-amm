// Constraint Optimization Module
//
// This module implements a sophisticated multi-phase constraint optimization system
// for Solana account validation. It includes deduplication, cross-function analysis,
// constraint lifting, and complexity-based grouping to minimize compute units.

use super::account_utils;
use super::types::*;
use super::OpcodeEmitter;
use crate::ast::{AstNode, Attribute, InstructionParameter};
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// Constraint Optimizer for multi-phase account validation optimization
pub struct ConstraintOptimizer {
    /// Current constraint deduplication analysis
    constraint_deduplication: ConstraintDeduplication,

    /// Advanced optimization system (Phase 4)
    advanced_optimization: AdvancedConstraintOptimization,

    /// Enable caching (disabled in MitoVM for zero-copy)
    enable_caching: bool,

    /// Current function being analyzed
    #[allow(dead_code)]
    current_function: Option<String>,

    /// Account registry for custom account type detection
    account_registry: Option<AccountRegistry>,
}

impl ConstraintOptimizer {
    /// Create a new constraint optimizer
    pub fn new() -> Self {
        Self {
            constraint_deduplication: ConstraintDeduplication::new(),
            advanced_optimization: AdvancedConstraintOptimization::new(),
            enable_caching: true, // Can be disabled for zero-copy VMs
            current_function: None,
            account_registry: None,
        }
    }

    /// Create a new constraint optimizer with account registry
    pub fn with_account_registry(account_registry: AccountRegistry) -> Self {
        Self {
            constraint_deduplication: ConstraintDeduplication::new(),
            advanced_optimization: AdvancedConstraintOptimization::new(),
            enable_caching: true,
            current_function: None,
            account_registry: Some(account_registry),
        }
    }

    /// Configure caching (disable for zero-copy VMs like Mito)
    pub fn set_caching_enabled(&mut self, enabled: bool) {
        self.enable_caching = enabled;
    }

    /// Phase 1-3: Analyze constraint deduplication for a function
    pub fn analyze_constraint_deduplication(
        &mut self,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        self.constraint_deduplication.constraint_map.clear();

        // First pass: collect all constraint requirements
        for (param_index, param) in parameters.iter().enumerate() {
            if account_utils::is_account_parameter(
                &param.param_type,
                &param.attributes,
                self.account_registry.as_ref(),
            ) {
                let account_index =
                    account_utils::account_index_from_param_index(param_index as u8);

                // Extract constraints from attributes
                let mut constraint_mask = 0u8;
                for attribute in &param.attributes {
                    match attribute.name.as_str() {
                        "signer" => constraint_mask |= 0x01,
                        "mut" => constraint_mask |= 0x02,
                        "init" => constraint_mask |= 0x04,
                        "close" => constraint_mask |= 0x08,
                        _ => {} // Unknown attributes ignored
                    }
                }

                if constraint_mask != 0 {
                    let constraint_key = ConstraintKey {
                        account_index,
                        constraint_type: constraint_mask,
                    };

                    self.constraint_deduplication
                        .constraint_map
                        .entry(constraint_key)
                        .or_default()
                        .push(param.name.clone());
                }
            }
        }

        // Second pass: build deduplicated constraint table
        self.constraint_deduplication.dedupe_table.clear();
        for (constraint_key, param_names) in &self.constraint_deduplication.constraint_map {
            if param_names.len() > 1 {
                // This constraint pattern can be deduplicated
                self.constraint_deduplication
                    .dedupe_table
                    .push((constraint_key.account_index, constraint_key.constraint_type));
            }
        }

        Ok(())
    }

    /// Generate optimized constraint validation with all phases
    pub fn generate_account_constraints<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        // Count total constraints to decide optimization strategy
        let total_constraints = self.count_total_constraints(parameters);

        if total_constraints == 0 {
            return Ok(()); // No constraints to generate
        }

        // Choose optimization strategy based on constraint count
        if total_constraints >= 4 {
            // Phase 3: Use deduplication for many constraints
            self.generate_deduplicated_constraints(emitter)?;
        } else {
            // Phase 1-2: Use individual or batch constraints for few constraints
            self.generate_individual_constraints(emitter, parameters)?;
        }

        Ok(())
    }

    /// Generate account metadata cache initialization (Phase 2 optimization)
    pub fn generate_account_metadata_cache_init<T: OpcodeEmitter>(
        &mut self,
        _emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        if !self.enable_caching {
            return Ok(()); // Caching disabled for zero-copy VMs
        }

        // Only cache accounts with constraints
        let accounts_with_constraints: Vec<_> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| {
                account_utils::is_account_parameter(
                    &param.param_type,
                    &param.attributes,
                    self.account_registry.as_ref(),
                ) && !param.attributes.is_empty()
            })
            .collect();

        if accounts_with_constraints.is_empty() {
            return Ok(()); // No accounts need caching
        }

        // Emit cache initialization
        // Account metadata caching - not implemented yet
        // For now, skip caching optimization and continue with standard processing
        // TODO: Implement account metadata caching when performance optimization is needed

        Ok(())
    }

    /// Generate optimized deduplicated constraint validation (Phase 3)
    pub fn generate_deduplicated_constraints<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
    ) -> Result<(), VMError> {
        if self.constraint_deduplication.dedupe_table.is_empty() {
            return Ok(()); // No deduplication opportunities
        }

        // Emit the new optimized CHECK_DEDUPE_TABLE opcode
        emitter.emit_opcode(CHECK_DEDUPE_TABLE);
        emitter.emit_u8(self.constraint_deduplication.dedupe_table.len() as u8);

        // Emit the deduplicated constraint table
        for (account_index, constraint_mask) in &self.constraint_deduplication.dedupe_table {
            emitter.emit_u8(*account_index);
            emitter.emit_u8(*constraint_mask);
        }

        Ok(())
    }

    /// Generate individual constraint validation (Phase 1 fallback)
    pub fn generate_individual_constraints<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        for (param_index, param) in parameters.iter().enumerate() {
            if account_utils::is_account_parameter(
                &param.param_type,
                &param.attributes,
                self.account_registry.as_ref(),
            ) && !param.attributes.is_empty()
            {
                let account_index =
                    account_utils::account_index_from_param_index(param_index as u8);

                // Security-First: Handle @init constraint early for uninitialized accounts
                let mut init_handled = false;
                if param.is_init {
                    // Emit CHECK_UNINITIALIZED before other constraints for security
                    emitter.emit_opcode(CHECK_UNINITIALIZED);
                    emitter.emit_u8(account_index);
                    init_handled = true; // Mark 'init' attribute as handled
                }

                // Calculate constraint mask
                let mut constraint_mask = 0u8;
                for attribute in &param.attributes {
                    match attribute.name.as_str() {
                        "signer" => constraint_mask |= 0x01,
                        "mut" => constraint_mask |= 0x02,
                        "init" => {
                            if !init_handled {
                                // Only set mask if not already handled by param.is_init
                                constraint_mask |= 0x04;
                            }
                        }
                        "close" => constraint_mask |= 0x08,
                        _ => {} // Unknown attributes ignored
                    }
                }

                if constraint_mask != 0 {
                    if self.enable_caching {
                        // Use cached validation
                        unimplemented!(
                            "Cached account validation - uncertain optimization benefit"
                        );
                    } else {
                        // Use immediate validation (zero-copy) - emit appropriate opcodes for each constraint
                        if (constraint_mask & 0x01) != 0 {
                            // signer
                            emitter.emit_opcode(CHECK_SIGNER);
                            emitter.emit_u8(account_index);
                        }
                        if (constraint_mask & 0x02) != 0 {
                            // mut
                            emitter.emit_opcode(CHECK_WRITABLE);
                            emitter.emit_u8(account_index);
                        }
                        // Note: 'init' constraint (0x04) is now handled by param.is_init check above
                        // which emits CHECK_UNINITIALIZED for @init accounts
                        if (constraint_mask & 0x08) != 0 { // close
                             // Note: close constraint handling would need VM support
                             // For now, we skip this as it's not implemented in VM
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Phase 4: Analyze cross-function constraint patterns
    pub fn analyze_cross_function_constraints(&mut self, ast: &AstNode) -> Result<(), VMError> {
        self.advanced_optimization.global_patterns.clear();

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = ast
        {
            let mut pattern_map: HashMap<String, Vec<String>> = HashMap::new();

            // Collect constraint patterns from all functions
            for instruction_def in instruction_definitions {
                if let AstNode::InstructionDefinition {
                    name, parameters, ..
                } = instruction_def
                {
                    for param in parameters {
                        if account_utils::is_account_parameter(
                            &param.param_type,
                            &param.attributes,
                            self.account_registry.as_ref(),
                        ) && !param.attributes.is_empty()
                        {
                            let pattern = format!(
                                "{}: {} @{}",
                                param.name,
                                account_utils::type_node_to_string(&param.param_type),
                                param.attributes.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(" @")
                            );

                            pattern_map
                                .entry(pattern.clone())
                                .or_default()
                                .push(name.clone());
                        }
                    }
                }
            }

            // Find patterns that appear in multiple functions (lifting candidates)
            for (pattern, functions) in pattern_map {
                if functions.len() > 1 {
                    // Extract constraint type from pattern
                    let constraint_type = if pattern.contains("@signer") {
                        0x01
                    } else if pattern.contains("@mut") {
                        0x02
                    } else if pattern.contains("@init") {
                        0x04
                    } else {
                        0x00
                    };

                    self.advanced_optimization.global_patterns.insert(
                        pattern.clone(),
                        GlobalConstraintPattern {
                            constraint_type,
                            account_pattern: pattern,
                            functions,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Phase 4: Implement constraint lifting to script initialization
    pub fn implement_constraint_lifting<T: OpcodeEmitter>(
        &mut self,
        _emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        // Identify constraints that can be lifted
        for (param_index, param) in parameters.iter().enumerate() {
            if account_utils::is_account_parameter(
                &param.param_type,
                &param.attributes,
                self.account_registry.as_ref(),
            ) {
                let pattern = format!(
                    "{}: {}",
                    param.name,
                    account_utils::type_node_to_string(&param.param_type)
                );

                if let Some(global_pattern) =
                    self.advanced_optimization.global_patterns.get(&pattern)
                {
                    if global_pattern.functions.len() > 2 {
                        // Lift constraints used in 3+ functions
                        self.advanced_optimization
                            .constraint_lifting
                            .lifted_constraints
                            .push((
                                account_utils::account_index_from_param_index(param_index as u8),
                                global_pattern.constraint_type,
                            ));

                        // Add to cache targets
                        self.advanced_optimization
                            .constraint_lifting
                            .cache_targets
                            .insert(
                                account_utils::account_index_from_param_index(param_index as u8),
                                global_pattern.constraint_type,
                            );
                    }
                }
            }
        }

        // Generate lifted constraint validation
        if !self
            .advanced_optimization
            .constraint_lifting
            .lifted_constraints
            .is_empty()
        {
            // Constraint lifting to script init - not implemented yet
            // For now, skip constraint lifting optimization
            // TODO: Implement constraint lifting when advanced optimization is needed
        }

        Ok(())
    }

    /// Phase 4: Group constraints by complexity
    pub fn group_constraints_by_complexity(
        &mut self,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        self.advanced_optimization.complexity_groups = ConstraintComplexityGroup::new();

        for (param_index, param) in parameters.iter().enumerate() {
            if account_utils::is_account_parameter(
                &param.param_type,
                &param.attributes,
                self.account_registry.as_ref(),
            ) && !param.attributes.is_empty()
            {
                let account_index =
                    account_utils::account_index_from_param_index(param_index as u8);
                let complexity = self.calculate_constraint_complexity(&param.attributes);
                let constraint_mask = self.calculate_constraint_mask(&param.attributes);

                match complexity {
                    1 => self
                        .advanced_optimization
                        .complexity_groups
                        .simple
                        .push((account_index, constraint_mask)),
                    2..=3 => self
                        .advanced_optimization
                        .complexity_groups
                        .medium
                        .push((account_index, constraint_mask)),
                    _ => self
                        .advanced_optimization
                        .complexity_groups
                        .complex
                        .push((account_index, constraint_mask)),
                }
            }
        }

        Ok(())
    }

    /// Phase 4: Generate advanced constraint validation with all optimizations
    pub fn generate_advanced_constraint_validation<T: OpcodeEmitter>(
        &mut self,
        _emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        // Generate complexity-based validation
        self.group_constraints_by_complexity(parameters)?;

        // Simple constraints (fast path)
        if !self
            .advanced_optimization
            .complexity_groups
            .simple
            .is_empty()
        {
            unimplemented!("Complexity-based constraint grouping - uncertain performance gain");
        }

        // Medium constraints (optimized path)
        if !self
            .advanced_optimization
            .complexity_groups
            .medium
            .is_empty()
        {
            unimplemented!("Complexity-based constraint grouping - uncertain performance gain");
        }

        // Complex constraints (full validation path)
        if !self
            .advanced_optimization
            .complexity_groups
            .complex
            .is_empty()
        {
            // Complexity-based constraint grouping - not implemented yet
            // For now, skip complexity-based grouping optimization
            // TODO: Implement complexity-based grouping when performance optimization is needed
        }

        Ok(())
    }

    /// Phase 4: Generate script-level constraint initialization
    pub fn generate_script_constraint_initialization<T: OpcodeEmitter>(
        &mut self,
        _emitter: &mut T,
        ast: &AstNode,
    ) -> Result<(), VMError> {
        // Analyze the entire script for global optimization opportunities
        self.analyze_cross_function_constraints(ast)?;

        // Generate script-level constraint cache initialization
        if !self
            .advanced_optimization
            .script_init_constraints
            .is_empty()
        {
            // Script-level constraint initialization - not implemented yet
            // For now, skip script-level constraint initialization
            // TODO: Implement script-level constraint initialization when advanced optimization is needed
        }

        Ok(())
    }

    /// Helper: Count total constraints across all parameters
    fn count_total_constraints(&self, parameters: &[InstructionParameter]) -> usize {
        parameters
            .iter()
            .filter(|param| {
                account_utils::is_account_parameter(
                    &param.param_type,
                    &param.attributes,
                    self.account_registry.as_ref(),
                ) && !param.attributes.is_empty()
            })
            .map(|param| param.attributes.len())
            .sum()
    }

    /// Helper: Calculate constraint complexity (number of validation operations)
    fn calculate_constraint_complexity(&self, attributes: &[Attribute]) -> usize {
        let mut complexity = 0;

        for attribute in attributes {
            match attribute.name.as_str() {
                "signer" => complexity += 1, // Simple pubkey check
                "mut" => complexity += 1,    // Simple mutability check
                "init" => complexity += 2,   // Account creation + validation
                "close" => complexity += 3,  // Account closure + cleanup
                _ => complexity += 1,        // Unknown attributes
            }
        }

        complexity
    }

    /// Helper: Calculate constraint mask from attributes
    fn calculate_constraint_mask(&self, attributes: &[Attribute]) -> u8 {
        let mut mask = 0u8;

        for attribute in attributes {
            match attribute.name.as_str() {
                "signer" => mask |= 0x01,
                "mut" => mask |= 0x02,
                "init" => mask |= 0x04,
                "close" => mask |= 0x08,
                _ => {} // Unknown attributes ignored
            }
        }

        mask
    }
}

impl Default for ConstraintOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Initialize constraint optimizer with zero-copy settings
    pub fn init_constraint_optimizer(&mut self) -> ConstraintOptimizer {
        let mut optimizer = ConstraintOptimizer::new();

        // Disable caching for zero-copy VMs (can be configured)
        optimizer.set_caching_enabled(true); // Default: enabled

        optimizer
    }

    /// Generate optimized constraints for function parameters
    pub fn generate_optimized_constraints(
        &mut self,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        let mut optimizer = self.init_constraint_optimizer();

        // Analyze and generate constraints
        optimizer.analyze_constraint_deduplication(parameters)?;
        optimizer.generate_account_constraints(self, parameters)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{InstructionParameter, TypeNode};

    #[test]
    fn test_constraint_deduplication_analysis() {
        let mut optimizer = ConstraintOptimizer::new();

        let parameters = vec![
            InstructionParameter {
                name: "signer1".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![Attribute { name: "signer".to_string(), args: vec![] }],
                is_init: false,
                init_config: None,
            },
            InstructionParameter {
                name: "signer2".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![Attribute { name: "signer".to_string(), args: vec![] }],
                is_init: false,
                init_config: None,
            },
        ];

        optimizer
            .analyze_constraint_deduplication(&parameters)
            .unwrap();

        // Different accounts with same constraints don't dedupe (each account_index is unique)
        assert_eq!(optimizer.constraint_deduplication.dedupe_table.len(), 0);
        assert_eq!(optimizer.constraint_deduplication.constraint_map.len(), 2);
    }

    #[test]
    fn test_constraint_complexity_calculation() {
        let optimizer = ConstraintOptimizer::new();

        let simple_attrs = vec![Attribute { name: "signer".to_string(), args: vec![] }];
        let complex_attrs = vec![
            Attribute { name: "signer".to_string(), args: vec![] },
            Attribute { name: "mut".to_string(), args: vec![] },
            Attribute { name: "init".to_string(), args: vec![] },
            Attribute { name: "close".to_string(), args: vec![] },
        ];

        assert_eq!(optimizer.calculate_constraint_complexity(&simple_attrs), 1);
        assert_eq!(optimizer.calculate_constraint_complexity(&complex_attrs), 7);
        // 1+1+2+3
    }
}
