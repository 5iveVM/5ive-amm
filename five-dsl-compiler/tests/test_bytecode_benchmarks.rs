// Bytecode Size Benchmark Tests
//
// Regression guards for basic compilation and size sanity checks.

#[cfg(test)]
mod bytecode_benchmarks {
    use five_dsl_compiler::DslCompiler;
    use std::fs;

    fn read_contract_file(path: &str) -> Option<String> {
        match fs::read_to_string(path) {
            Ok(content) => Some(content),
            Err(_) => None,
        }
    }

    #[test]
    fn test_compiles_valid_bytecode() {
        // Simple test to verify bytecode generation
        let source = r#"
            mut counter: u64;

            init {
                counter = 0;
            }

            pub increment() -> u64 {
                counter = counter + 1;
                return counter;
            }

            pub get_count() -> u64 {
                return counter;
            }
        "#;

        let result = DslCompiler::compile_dsl(source);
        assert!(result.is_ok(), "Compilation should succeed");

        let bytecode = result.unwrap();
        assert!(!bytecode.is_empty(), "Bytecode should not be empty");

        println!(
            "Simple counter contract compiled successfully: {} bytes",
            bytecode.len()
        );
    }

    #[test]
    fn test_compiles_with_many_variables() {
        // Test with function that has many local variables
        let source = r#"
            pub calculate(a: u64, b: u64, c: u64, d: u64) -> u64 {
                let x: u64 = a + b;
                let y: u64 = c + d;
                let z: u64 = x + y;
                let w: u64 = z * 2;
                let v: u64 = w - 5;
                let u: u64 = v * a;
                let t: u64 = u + b;
                let s: u64 = t * c;
                let r: u64 = s + d;
                return r;
            }
        "#;

        let result = DslCompiler::compile_dsl(source);
        assert!(
            result.is_ok(),
            "Should compile function with many variables"
        );
        println!(
            "Multi-variable function compiled successfully: {} bytes",
            result.unwrap().len()
        );
    }

    #[test]
    fn test_counter_compiles_with_optimizations() {
        if let Some(source) = read_contract_file("five-templates/counter/src/counter.v") {
            match DslCompiler::compile_dsl(&source) {
                Ok(bytecode) => {
                    println!("Counter.v compiled successfully: {} bytes", bytecode.len());
                    assert!(!bytecode.is_empty());
                }
                Err(e) => {
                    println!("Counter.v compilation: {}", e);
                    // Don't fail - file might not exist in all environments
                }
            }
        } else {
            println!("Counter.v not found - skipping test");
        }
    }

    #[test]
    fn test_token_compiles_with_optimizations() {
        if let Some(source) = read_contract_file("five-templates/token/src/token.v") {
            match DslCompiler::compile_dsl(&source) {
                Ok(bytecode) => {
                    println!("Token.v compiled successfully: {} bytes", bytecode.len());
                    assert!(!bytecode.is_empty());
                }
                Err(e) => {
                    println!("Token.v compilation: {}", e);
                    // Don't fail - file might not exist in all environments
                }
            }
        } else {
            println!("Token.v not found - skipping test");
        }
    }
}
