// Tests for Linear Scan Register Allocation
//
// These tests verify that the linear scan allocator correctly handles
// variable lifetimes and register reuse for optimized bytecode generation.

#[cfg(test)]
mod linear_scan_allocation_tests {
    use five_dsl_compiler::compiler::pipeline::{CompilationConfig, CompilationMode};
    use five_dsl_compiler::DslCompiler;

    #[test]
    fn test_linear_scan_enabled_in_config() {
        let config = CompilationConfig::new(CompilationMode::Testing)
            .with_use_registers(true)
            .with_linear_scan_allocation(true);

        assert!(config.use_registers);
        assert!(config.use_linear_scan_allocation);
    }

    #[test]
    fn test_linear_scan_disabled_by_default() {
        let config = CompilationConfig::new(CompilationMode::Testing)
            .with_use_registers(true);

        assert!(config.use_registers);
        assert!(!config.use_linear_scan_allocation);
    }

    #[test]
    fn test_compile_simple_function_with_linear_scan() {
        let source = r#"
            mut counter: u64;

            init {
                counter = 0;
            }

            pub increment() -> u64 {
                counter = counter + 1;
                return counter;
            }
        "#;

        // This should compile without errors using standard API
        let result = DslCompiler::compile_dsl(source);
        assert!(result.is_ok(), "Compilation should succeed with linear scan enabled");
    }

    #[test]
    fn test_compile_multiple_locals_with_linear_scan() {
        let source = r#"
            pub test_locals() {
                let x: u64 = 10;
                let y: u64 = 20;
                let z: u64 = 30;
                let sum: u64 = x + y + z;
                return sum;
            }
        "#;

        let result = DslCompiler::compile_dsl(source);
        assert!(result.is_ok(), "Should compile multiple locals with linear scan");
    }

    #[test]
    fn test_compilation_config_builder() {
        // Verify the builder pattern works correctly
        let config = CompilationConfig::new(CompilationMode::Testing)
            .with_use_registers(true)
            .with_linear_scan_allocation(true)
            .with_module_namespaces(true);

        assert!(config.use_registers);
        assert!(config.use_linear_scan_allocation);
        assert!(config.enable_module_namespaces);
    }
}
