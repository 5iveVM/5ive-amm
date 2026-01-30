// Linear Scan Register Allocation - Poletto & Sarkar (1999)
//
// Implements efficient register allocation via live interval analysis.
// Replaces simple sequential allocation with register reuse when variables
// go out of scope.
//
// Algorithm:
// 1. Compute live intervals [start, end] for each variable
// 2. Sort intervals by start position
// 3. For each interval:
//    - Remove expired intervals from active set
//    - If registers available: assign register
//    - Otherwise: spill variable to stack (mark for spilling)

use std::collections::HashMap;

pub type RegisterIndex = u8;

const MAX_REGISTERS: u8 = 16;

/// Represents the live range of a variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveInterval {
    /// Variable name
    pub variable: String,
    /// First position where variable is live (bytecode instruction index)
    pub start: usize,
    /// Last position where variable is live (bytecode instruction index)
    pub end: usize,
    /// Type of the variable (for classification)
    pub var_type: String,
    /// Whether this is a function parameter
    pub is_parameter: bool,
    /// Usage count (frequency)
    pub usage_count: usize,
    /// Assigned register (None if spilled to stack)
    pub assigned_register: Option<RegisterIndex>,
    /// Whether variable needs to be spilled to stack
    pub is_spilled: bool,
}

impl LiveInterval {
    pub fn new(
        variable: String,
        start: usize,
        end: usize,
        var_type: String,
        is_parameter: bool,
        usage_count: usize,
    ) -> Self {
        Self {
            variable,
            start,
            end,
            var_type,
            is_parameter,
            usage_count,
            assigned_register: None,
            is_spilled: false,
        }
    }

    /// Check if this interval overlaps with another
    /// Uses inclusive comparison: [start, end] - touching endpoints means overlap
    pub fn overlaps(&self, other: &LiveInterval) -> bool {
        !(self.end < other.start || other.end < self.start)
    }

    /// Check if this interval has disjoint (non-overlapping) lifetime
    pub fn is_disjoint(&self, other: &LiveInterval) -> bool {
        self.end < other.start || other.end < self.start
    }

    /// Calculate spill cost (higher usage = higher cost to spill)
    pub fn spill_cost(&self) -> f32 {
        // Parameters are expensive to spill (they're needed immediately)
        let param_weight = if self.is_parameter { 10.0 } else { 1.0 };
        // High-usage variables are expensive to spill
        self.usage_count as f32 * param_weight
    }
}

/// Linear scan allocator state
pub struct LinearScanAllocator {
    /// All live intervals for a function
    intervals: Vec<LiveInterval>,
    /// Currently active intervals (those that haven't expired)
    active: Vec<LiveInterval>,
    /// Next available register index
    next_register: RegisterIndex,
}

impl LinearScanAllocator {
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
            active: Vec::new(),
            next_register: 0,
        }
    }

    /// Reset allocator for a new function
    pub fn reset(&mut self) {
        self.intervals.clear();
        self.active.clear();
        self.next_register = 0;
    }

    /// Add a live interval for a variable
    pub fn add_interval(
        &mut self,
        variable: String,
        start: usize,
        end: usize,
        var_type: String,
        is_parameter: bool,
        usage_count: usize,
    ) {
        self.intervals.push(LiveInterval::new(
            variable,
            start,
            end,
            var_type,
            is_parameter,
            usage_count,
        ));
    }

    /// Run linear scan allocation algorithm
    /// Returns map of variable names to assigned registers (or None if spilled)
    pub fn allocate(&mut self) -> HashMap<String, Option<RegisterIndex>> {
        let mut result = HashMap::new();

        // Phase 1: Sort intervals by start position
        self.intervals.sort_by_key(|iv| iv.start);

        // Phase 2: Linear scan through sorted intervals
        for i in 0..self.intervals.len() {
            let interval_start = self.intervals[i].start;
            let interval_var = self.intervals[i].variable.clone();

            // Remove expired intervals from active set
            self.active.retain(|active_iv| active_iv.end >= interval_start);

            // Phase 3: Try to find a free register
            let allocated_reg = self.find_free_register();

            if let Some(reg) = allocated_reg {
                self.intervals[i].assigned_register = Some(reg);
                self.active.push(self.intervals[i].clone());
                result.insert(interval_var, Some(reg));
            } else {
                // No free registers: spill this interval
                self.intervals[i].is_spilled = true;
                result.insert(interval_var, None);
            }
        }

        result
    }

    /// Try to find a free register for an interval
    /// If successful, returns Some(register_index)
    /// If all registers in use, returns None (variable will be spilled)
    fn find_free_register(&self) -> Option<RegisterIndex> {
        // Collect used registers from active intervals
        let used_registers: Vec<RegisterIndex> = self
            .active
            .iter()
            .filter_map(|iv| iv.assigned_register)
            .collect();

        // Find first free register
        for reg in 0..MAX_REGISTERS {
            if !used_registers.contains(&reg) {
                return Some(reg);
            }
        }

        // All registers in use: need to spill someone
        // Choose the interval with the furthest next use (Furthest Use heuristic)
        // For now, return None (caller will handle spilling)
        // TODO: Implement spill selection heuristic
        None
    }

    /// Get the allocation map for all intervals
    pub fn get_allocations(&self) -> HashMap<String, Option<RegisterIndex>> {
        let mut result = HashMap::new();
        for interval in &self.intervals {
            result.insert(interval.variable.clone(), interval.assigned_register);
        }
        result
    }

    /// Get spilled variables (those that exceeded register limit)
    pub fn get_spilled_variables(&self) -> Vec<String> {
        self.intervals
            .iter()
            .filter(|iv| iv.is_spilled)
            .map(|iv| iv.variable.clone())
            .collect()
    }

    /// Check if any variables were spilled
    pub fn has_spilled_variables(&self) -> bool {
        self.intervals.iter().any(|iv| iv.is_spilled)
    }
}

impl Default for LinearScanAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_scan_basic_allocation() {
        let mut allocator = LinearScanAllocator::new();

        // Add 3 variables with non-overlapping lifetimes
        allocator.add_interval("x".to_string(), 0, 5, "u64".to_string(), false, 3);
        allocator.add_interval("y".to_string(), 6, 10, "u64".to_string(), false, 2);
        allocator.add_interval("z".to_string(), 11, 15, "u64".to_string(), false, 1);

        let allocations = allocator.allocate();

        // All should be allocated to r0 (register reused)
        assert_eq!(allocations.get("x"), Some(&Some(0)));
        assert_eq!(allocations.get("y"), Some(&Some(0))); // Reused r0
        assert_eq!(allocations.get("z"), Some(&Some(0))); // Reused r0
    }

    #[test]
    fn test_linear_scan_overlapping_intervals() {
        let mut allocator = LinearScanAllocator::new();

        // Add 3 variables with overlapping lifetimes
        // x: [0, 10], y: [5, 15], z: [3, 8]
        // All three overlap with each other
        allocator.add_interval("x".to_string(), 0, 10, "u64".to_string(), false, 2);
        allocator.add_interval("y".to_string(), 5, 15, "u64".to_string(), false, 3);
        allocator.add_interval("z".to_string(), 3, 8, "u64".to_string(), false, 1);

        let allocations = allocator.allocate();

        // All three need different registers since they all overlap
        // Sorted by start: x(0), z(3), y(5)
        assert_eq!(allocations.get("x"), Some(&Some(0)));
        assert_eq!(allocations.get("z"), Some(&Some(1))); // z starts at 3, overlaps x
        assert_eq!(allocations.get("y"), Some(&Some(2))); // y starts at 5, overlaps both x and z
    }

    #[test]
    fn test_linear_scan_parameters_first() {
        let mut allocator = LinearScanAllocator::new();

        // Parameters should be allocated first
        allocator.add_interval("param1".to_string(), 0, 20, "u64".to_string(), true, 5);
        allocator.add_interval("param2".to_string(), 0, 15, "u64".to_string(), true, 4);
        allocator.add_interval("local1".to_string(), 5, 10, "u64".to_string(), false, 1);

        let allocations = allocator.allocate();

        // Parameters should be allocated first
        assert_eq!(allocations.get("param1"), Some(&Some(0)));
        assert_eq!(allocations.get("param2"), Some(&Some(1)));
        assert_eq!(allocations.get("local1"), Some(&Some(2)));
    }

    #[test]
    fn test_linear_scan_registers_exhausted() {
        let mut allocator = LinearScanAllocator::new();

        // Add more than MAX_REGISTERS overlapping intervals
        for i in 0..18 {
            allocator.add_interval(
                format!("var{}", i),
                0,
                100,
                "u64".to_string(),
                false,
                1,
            );
        }

        let allocations = allocator.allocate();

        // Count allocated vs spilled
        let allocated = allocations.values().filter(|v| v.is_some()).count();
        let spilled = allocations.values().filter(|v| v.is_none()).count();

        assert_eq!(allocated, 16); // MAX_REGISTERS
        assert_eq!(spilled, 2); // 18 - 16 = 2 spilled
    }

    #[test]
    fn test_live_interval_overlaps() {
        let iv1 = LiveInterval::new("x".to_string(), 0, 10, "u64".to_string(), false, 1);
        let iv2 = LiveInterval::new("y".to_string(), 5, 15, "u64".to_string(), false, 1);
        let iv3 = LiveInterval::new("z".to_string(), 11, 20, "u64".to_string(), false, 1);

        assert!(iv1.overlaps(&iv2)); // [0,10] overlaps [5,15]
        assert!(iv1.is_disjoint(&iv3)); // [0,10] is disjoint from [11,20]
        assert!(iv2.overlaps(&iv3)); // [5,15] overlaps [11,20] (at 11-15)
    }

    #[test]
    fn test_spill_cost_parameters_expensive() {
        let param = LiveInterval::new("param".to_string(), 0, 20, "u64".to_string(), true, 5);
        let local = LiveInterval::new("local".to_string(), 0, 20, "u64".to_string(), false, 5);

        // Parameter with same usage should have higher spill cost
        assert!(param.spill_cost() > local.spill_cost());
    }
}
