// Register Allocator - Hybrid Allocation Strategy
//
// Manages register allocation for function parameters and local variables.
// Variables are mapped to persistent registers (not temporary scratch space).
//
// Allocation Strategies:
// 1. Sequential (Default) - Simple O(1) allocation, no register reuse
// 2. Linear Scan (Opt-in) - Reuses registers when variables go out of scope
//    Uses Poletto & Sarkar (1999) live interval analysis
//
// Model:
// - Function parameters are mapped to r0, r1, ... at function entry
// - Local variables are mapped to registers as they're first assigned
// - With linear scan: registers freed when variable's live range ends
// - Without linear scan: variables stay in registers throughout function scope

use std::collections::HashMap;
use crate::bytecode_generator::linear_scan_allocator::{LinearScanAllocator, LiveInterval};

/// Represents a VM register index (0-15)
pub type RegisterIndex = u8;

const MAX_REGISTERS: u8 = 16; // VM supports r0-r15

pub struct RegisterAllocator {
    /// Maps variable names to their register index
    /// Example: "amount" -> 0, "sender" -> 1, "balance" -> 2
    variable_to_register: HashMap<String, RegisterIndex>,
    /// Next available register for new variable allocations (sequential mode)
    next_register: RegisterIndex,
    /// Linear scan allocator (for advanced allocation)
    linear_scan: LinearScanAllocator,
    /// Whether to use linear scan allocation
    use_linear_scan: bool,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            variable_to_register: HashMap::new(),
            next_register: 0,
            linear_scan: LinearScanAllocator::new(),
            use_linear_scan: false,
        }
    }

    /// Create with linear scan allocation enabled
    pub fn with_linear_scan() -> Self {
        Self {
            variable_to_register: HashMap::new(),
            next_register: 0,
            linear_scan: LinearScanAllocator::new(),
            use_linear_scan: true,
        }
    }

    /// Enable or disable linear scan allocation
    pub fn set_use_linear_scan(&mut self, enabled: bool) {
        self.use_linear_scan = enabled;
    }

    /// Map a function parameter to a register at function entry
    ///
    /// Called sequentially for each parameter:
    /// map_parameter("amount", 0) -> r0
    /// map_parameter("sender", 1) -> r1
    /// etc.
    pub fn map_parameter(&mut self, name: &str, param_idx: RegisterIndex) -> Option<RegisterIndex> {
        if param_idx >= MAX_REGISTERS {
            return None;
        }
        self.variable_to_register.insert(name.to_string(), param_idx);
        // Update next_register if this parameter is the highest so far
        if param_idx >= self.next_register {
            self.next_register = param_idx + 1;
        }
        Some(param_idx)
    }

    /// Map a local variable to the next available register
    ///
    /// Called when a local variable is first assigned:
    /// let x = 5;  -> map_local("x") -> r2 (if parameters took r0, r1)
    pub fn map_local(&mut self, name: &str) -> Option<RegisterIndex> {
        // Don't remap if already exists
        if self.variable_to_register.contains_key(name) {
            return self.variable_to_register.get(name).copied();
        }

        if self.next_register >= MAX_REGISTERS {
            return None; // Out of registers
        }

        let reg = self.next_register;
        self.variable_to_register.insert(name.to_string(), reg);
        self.next_register += 1;
        Some(reg)
    }

    /// Get the register index for a variable (parameter or local)
    /// Returns None if variable is not mapped to a register
    pub fn get_mapping(&self, name: &str) -> Option<RegisterIndex> {
        let result = self.variable_to_register.get(name).copied();

        #[cfg(debug_assertions)]
        {
            if result.is_some() {
                println!("DEBUG: get_mapping('{}') found: r{}", name, result.unwrap());
            } else {
                println!("DEBUG: get_mapping('{}') NOT found. Available mappings: {} total",
                    name, self.variable_to_register.len());
                if self.variable_to_register.len() > 0 {
                    println!("  Available: {:?}",
                        self.variable_to_register.keys().map(|k| k.as_str()).collect::<Vec<_>>());
                }
            }
        }

        result
    }

    /// Check if a variable has been mapped to a register
    pub fn has_mapping(&self, name: &str) -> bool {
        self.variable_to_register.contains_key(name)
    }

    /// Reset the allocator for a new function
    pub fn reset(&mut self) {
        self.variable_to_register.clear();
        self.next_register = 0;
        self.linear_scan.reset();
    }

    /// Finalize and compute register allocations using linear scan
    /// Call this after all intervals have been added
    pub fn finalize_linear_scan(&mut self) -> HashMap<String, Option<RegisterIndex>> {
        self.linear_scan.allocate()
    }

    /// Add a live interval for linear scan allocation
    pub fn add_live_interval(
        &mut self,
        variable: String,
        start: usize,
        end: usize,
        var_type: String,
        is_parameter: bool,
        usage_count: usize,
    ) {
        self.linear_scan.add_interval(variable, start, end, var_type, is_parameter, usage_count);
    }

    /// Get spilled variables (for linear scan)
    pub fn get_spilled_variables(&self) -> Vec<String> {
        self.linear_scan.get_spilled_variables()
    }

    /// Check if any variables were spilled
    pub fn has_spilled_variables(&self) -> bool {
        self.linear_scan.has_spilled_variables()
    }

    /// Get the number of registers currently in use
    pub fn registers_in_use(&self) -> u8 {
        self.next_register
    }

    /// Get all variable-to-register mappings (for debugging)
    pub fn get_all_mappings(&self) -> &HashMap<String, RegisterIndex> {
        &self.variable_to_register
    }
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_mapping() {
        let mut allocator = RegisterAllocator::new();

        // Map parameters in order
        assert_eq!(allocator.map_parameter("amount", 0), Some(0));
        assert_eq!(allocator.map_parameter("sender", 1), Some(1));
        assert_eq!(allocator.map_parameter("payer", 2), Some(2));

        // Verify they're retrievable
        assert_eq!(allocator.get_mapping("amount"), Some(0));
        assert_eq!(allocator.get_mapping("sender"), Some(1));
        assert_eq!(allocator.get_mapping("payer"), Some(2));
    }

    #[test]
    fn test_local_variable_mapping() {
        let mut allocator = RegisterAllocator::new();

        // Map parameters first
        allocator.map_parameter("amount", 0).unwrap();
        allocator.map_parameter("sender", 1).unwrap();

        // Now map local variables
        assert_eq!(allocator.map_local("balance"), Some(2));
        assert_eq!(allocator.map_local("fee"), Some(3));

        // Verify mappings
        assert_eq!(allocator.get_mapping("balance"), Some(2));
        assert_eq!(allocator.get_mapping("fee"), Some(3));
    }

    #[test]
    fn test_register_exhaustion() {
        let mut allocator = RegisterAllocator::new();

        // Allocate all 16 registers
        for i in 0..MAX_REGISTERS {
            let name = format!("var{}", i);
            assert_eq!(allocator.map_parameter(&name, i), Some(i));
        }

        // Next allocation should fail
        assert_eq!(allocator.map_local("overflow"), None);
    }

    #[test]
    fn test_reset() {
        let mut allocator = RegisterAllocator::new();

        allocator.map_parameter("param", 0).unwrap();
        allocator.map_local("local").unwrap();

        assert_eq!(allocator.get_mapping("param"), Some(0));
        assert_eq!(allocator.get_mapping("local"), Some(1));

        // Reset
        allocator.reset();

        assert_eq!(allocator.get_mapping("param"), None);
        assert_eq!(allocator.get_mapping("local"), None);
        assert_eq!(allocator.registers_in_use(), 0);
    }

    #[test]
    fn test_duplicate_local_mapping() {
        let mut allocator = RegisterAllocator::new();

        allocator.map_parameter("param", 0).unwrap();

        // First local gets r1
        assert_eq!(allocator.map_local("x"), Some(1));

        // Remapping same local returns same register
        assert_eq!(allocator.map_local("x"), Some(1));

        // Next local gets r2
        assert_eq!(allocator.map_local("y"), Some(2));
    }

    #[test]
    fn test_linear_scan_allocator_enabled() {
        let mut allocator = RegisterAllocator::with_linear_scan();
        assert!(allocator.use_linear_scan);

        // Add live intervals
        allocator.add_live_interval(
            "x".to_string(),
            0,
            5,
            "u64".to_string(),
            false,
            3,
        );
        allocator.add_live_interval(
            "y".to_string(),
            6,
            10,
            "u64".to_string(),
            false,
            2,
        );

        // Finalize and check allocations
        let allocations = allocator.finalize_linear_scan();
        assert_eq!(allocations.get("x"), Some(&Some(0)));
        // y should reuse r0 (no overlap with x)
        assert_eq!(allocations.get("y"), Some(&Some(0)));
    }
}
