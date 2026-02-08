//! Resource tracking for header generation.

use super::types::ASTGenerator;

impl ASTGenerator {
    /// Track local variable usage for resource requirements
    pub fn track_local_variable(&mut self, _variable_name: &str) {
        self.max_locals_used = self.max_locals_used.saturating_add(1);
    }

    /// Track function call for call depth analysis
    pub fn track_function_call(&mut self) {
        self.current_call_depth = self.current_call_depth.saturating_add(1);
        self.max_call_depth_seen = self.max_call_depth_seen.max(self.current_call_depth);
        self.function_call_count = self.function_call_count.saturating_add(1);
    }

    /// Track function return for call depth analysis
    pub fn track_function_return(&mut self) {
        self.current_call_depth = self.current_call_depth.saturating_sub(1);
    }

    /// Track function parameters for stack resource calculation
    pub fn track_function_parameters(&mut self, param_count: u8) {
        // Each parameter uses stack space - account for worst case
        // This represents the stack space needed for parameters at call time
        let param_stack_usage = param_count as u16;
        self.max_stack_depth_seen = self.max_stack_depth_seen.saturating_add(param_stack_usage);

        // Also account for call frame overhead (return address + call metadata)
        // Estimate ~3 stack slots per call (return address, saved locals count, saved IP)
        self.max_stack_depth_seen = self.max_stack_depth_seen.saturating_add(3);
    }

    /// Track local variables used within a function for resource calculation
    pub fn track_function_locals(&mut self, _function_name: &str, local_count: u8) {
        // Track locals for called functions - this helps ensure we allocate enough
        // space for local variables used within function call chains
        self.max_locals_used = self.max_locals_used.max(local_count);

        // Each local variable may also use stack space for intermediate calculations
        // Conservative estimate: each local might contribute to stack usage
        let locals_stack_contribution = (local_count / 2) as u16; // Conservative estimate
        self.max_stack_depth_seen = self
            .max_stack_depth_seen
            .saturating_add(locals_stack_contribution);
    }

    /// Track string literal for resource estimation
    pub fn track_string_literal(&mut self, _content: &str) {
        self.string_literals_count = self.string_literals_count.saturating_add(1);
    }

    /// Track stack operations for depth analysis
    pub fn track_stack_operation(&mut self, stack_effect: i16) {
        // Note: This is a simplified tracking - real implementation would need
        // more sophisticated stack depth analysis during code generation
        if stack_effect > 0 {
            self.max_stack_depth_seen = self
                .max_stack_depth_seen
                .saturating_add(stack_effect as u16);
        }
    }

    /// Track operations that require temporary buffers
    pub fn track_temp_buffer_usage(&mut self, size_needed: u8) {
        self.estimated_temp_usage = self.estimated_temp_usage.max(size_needed);
    }

    /// Generate resource requirements from tracked data
    /// 🚀 PRODUCTION: Returns compile-time defaults for optimal performance
    pub fn generate_resource_requirements(&self) -> five_protocol::ResourceRequirements {
        five_protocol::ResourceRequirements {
            max_stack: self.max_stack_depth_seen as u32,
            max_memory: 0,
            max_locals: self.max_locals_used,
            max_stack_depth: self.max_stack_depth_seen,
            string_pool_bytes: self.string_literals_count,
            max_call_depth: self.max_call_depth_seen,
            temp_buffer_size: self.estimated_temp_usage,
            heap_string_capacity: 0,
            heap_array_capacity: 0,
        }
    }

    /// Reset resource tracking (for reusing generator)
    pub fn reset_resource_tracking(&mut self) {
        self.max_locals_used = 0;
        self.max_stack_depth_seen = 0;
        self.current_call_depth = 0;
        self.max_call_depth_seen = 1;
        self.string_literals_count = 0;
        self.estimated_temp_usage = 64;
        self.function_call_count = 0;
    }

    /// Get enhanced resource requirements using function call tracking and cross-function analysis
    /// 🚀 PRODUCTION: Returns compile-time defaults for optimal performance
    pub fn get_enhanced_resource_requirements(&self) -> five_protocol::ResourceRequirements {
        self.generate_resource_requirements()
    }

    /// Get the number of function calls tracked during compilation
    /// This provides insight into call complexity for resource allocation decisions.
    pub fn get_function_call_count(&self) -> u16 {
        self.function_call_count
    }
}
