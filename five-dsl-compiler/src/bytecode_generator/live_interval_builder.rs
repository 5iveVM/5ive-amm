// Live Interval Builder - Bridge between Scope Analysis and Linear Scan Allocation
//
// Takes variable scope information from ScopeAnalyzer and converts it into
// precise live intervals for Linear Scan Register Allocation.

use crate::bytecode_generator::scope_analyzer::{FunctionScopeAnalysis, VariableScope};
use crate::bytecode_generator::linear_scan_allocator::{LiveInterval, LinearScanAllocator};
use std::collections::HashMap;

/// Builds live intervals from function scope analysis
pub struct LiveIntervalBuilder;

impl LiveIntervalBuilder {
    /// Convert scope analysis into live intervals for linear scan allocation
    ///
    /// This processes the scope analysis output and creates precise live intervals
    /// that can be used by the linear scan allocator.
    pub fn build_intervals_from_scope_analysis(
        analysis: &FunctionScopeAnalysis,
    ) -> LinearScanAllocator {
        let mut allocator = LinearScanAllocator::new();

        // Convert each variable scope into a live interval
        for var_scope in &analysis.variables {
            let interval = Self::scope_to_interval(var_scope);
            allocator.add_interval(
                interval.variable.clone(),
                interval.start,
                interval.end,
                interval.var_type.clone(),
                interval.is_parameter,
                interval.usage_count,
            );
        }

        allocator
    }

    /// Convert a single VariableScope to a LiveInterval
    fn scope_to_interval(var_scope: &VariableScope) -> LiveInterval {
        LiveInterval::new(
            var_scope.name.clone(),
            var_scope.first_use,
            var_scope.last_use,
            var_scope.var_type.clone(),
            var_scope.is_parameter,
            var_scope.usage_count,
        )
    }

    /// Refine live intervals with additional heuristics
    ///
    /// Applies optimization techniques to improve register allocation:
    /// - Extend parameter intervals to end of function (they're always needed)
    /// - Mark hot variables for priority allocation
    pub fn refine_intervals(
        mut allocator: LinearScanAllocator,
        _max_position: usize,
    ) -> LinearScanAllocator {
        // TODO: Implement refinement strategies
        // - Extend parameter lifetimes to function end
        // - Detect loop-based extensions
        // - Mark hot vs cold variables
        allocator
    }

    /// Extract variable priority hints for allocation
    ///
    /// Returns a map of variable names to priority scores for priority-based allocation.
    pub fn extract_priority_hints(analysis: &FunctionScopeAnalysis) -> HashMap<String, u32> {
        let mut priorities = HashMap::new();

        for var_scope in &analysis.variables {
            // Priority = base + usage frequency + parameter bonus
            let mut priority = 0u32;

            if var_scope.is_parameter {
                // Parameters get base priority (always allocated first)
                priority += 1000;
            }

            // Usage frequency bonus (high-use variables get better registers)
            priority += var_scope.usage_count as u32 * 100;

            priorities.insert(var_scope.name.clone(), priority);
        }

        priorities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_to_interval_conversion() {
        let var_scope = VariableScope {
            name: "test_var".to_string(),
            var_type: "u64".to_string(),
            scope_level: 0,
            first_use: 5,
            last_use: 15,
            assigned_slot: None,
            usage_count: 3,
            is_parameter: false,
        };

        let interval = LiveIntervalBuilder::scope_to_interval(&var_scope);

        assert_eq!(interval.variable, "test_var");
        assert_eq!(interval.start, 5);
        assert_eq!(interval.end, 15);
        assert_eq!(interval.var_type, "u64");
        assert_eq!(interval.usage_count, 3);
        assert!(!interval.is_parameter);
    }

    #[test]
    fn test_priority_hints_parameters_high() {
        let var_scope_param = VariableScope {
            name: "param".to_string(),
            var_type: "u64".to_string(),
            scope_level: 0,
            first_use: 0,
            last_use: 20,
            assigned_slot: None,
            usage_count: 5,
            is_parameter: true,
        };

        let var_scope_local = VariableScope {
            name: "local".to_string(),
            var_type: "u64".to_string(),
            scope_level: 0,
            first_use: 5,
            last_use: 15,
            assigned_slot: None,
            usage_count: 5,
            is_parameter: false,
        };

        let analysis = FunctionScopeAnalysis {
            function_name: "test".to_string(),
            variables: vec![var_scope_param, var_scope_local],
            max_scope_level: 0,
            total_local_slots: 2,
            scope_stack: vec![],
        };

        let priorities = LiveIntervalBuilder::extract_priority_hints(&analysis);

        // Parameter should have higher priority despite same usage count
        assert!(priorities.get("param").unwrap() > priorities.get("local").unwrap());
    }
}
