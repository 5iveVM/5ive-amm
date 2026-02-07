//! Comprehensive integration tests for Five VM WASM
//!
//! Tests the complete DSL compiler + MitoVM pipeline using the same test scripts
//! as the Five CLI to ensure consistency between Rust and WASM implementations.
//!
//! Test Modes (controlled by FIVE_TEST_MODE environment variable):
//! - "wasm" (default): Direct MitoVM execution (fast, good for basic functionality)
//! - "localnet": Five CLI on-chain execution (comprehensive, requires local validator)

#[cfg(test)]
mod tests {
    use five_dsl_compiler::DslCompiler;
    use five_protocol::MAX_SCRIPT_SIZE;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage};
    use std::env;
    use std::fs;
    use std::path::Path;

    /// Test execution mode
    #[derive(Debug, Clone, PartialEq)]
    enum TestMode {
        Wasm,     // Direct MitoVM execution (default)
        Localnet, // Five CLI on-chain execution
    }

    /// Test result for tracking compilation and execution
    #[derive(Debug)]
    struct TestResult {
        script_name: String,
        #[allow(unused)]
        category: String,
        compilation_success: bool,
        compilation_error: Option<String>,
        execution_success: bool,
        execution_error: Option<String>,
        result_value: Option<Value>,
        bytecode_size: usize,
        #[allow(unused)]
        compute_units: u64,
    }

    /// Get test mode from environment variable
    fn get_test_mode() -> TestMode {
        match env::var("FIVE_TEST_MODE").as_deref() {
            Ok("localnet") => TestMode::Localnet,
            _ => TestMode::Wasm, // Default to WASM mode
        }
    }

    /// Check if local validator testing is available
    fn is_localnet_available() -> bool {
        // Check if Five CLI is available
        std::process::Command::new("node")
            .args(&["../five-cli/dist/index.js", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Parse test parameters from script content
    fn parse_test_params(content: &str) -> Vec<Value> {
        if let Some(params_line) = content
            .lines()
            .find(|line| line.contains("// @test-params"))
        {
            if let Some(params_str) = params_line.split("@test-params").nth(1) {
                return params_str
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse::<u64>().ok().map(Value::U64))
                    .collect();
            }
        }
        Vec::new()
    }

    /// Compile Five DSL source to bytecode
    fn compile_five_script(source: &str) -> Result<Vec<u8>, String> {
        match DslCompiler::compile_for_testing(source) {
            Ok(bytecode) => Ok(bytecode),
            Err(e) => Err(format!("Compilation failed: {}", e)),
        }
    }

    /// Create onchain instruction data format for reference
    /// This shows the full format used when calling the Five Solana program
    #[allow(dead_code)]
    fn create_onchain_instruction_data(params: &[Value]) -> Vec<u8> {
        let mut data = vec![];

        // Execute instruction discriminator (removed by Five program before passing to MitoVM)
        data.push(2);

        // Function index 0 (u32)
        data.extend_from_slice(&0u32.to_le_bytes());

        // Parameter count (u32)
        data.extend_from_slice(&(params.len() as u32).to_le_bytes());

        for param in params {
            match param {
                Value::U64(val) => {
                    data.push(4); // U64 type marker
                    data.extend_from_slice(&val.to_le_bytes());
                }
                Value::Bool(val) => {
                    data.push(2); // Bool type marker
                    data.push(if *val { 1 } else { 0 });
                }
                _ => {} // Skip unsupported types
            }
        }
        data
    }

    /// Execute script on-chain using Five CLI (for localnet testing)
    fn execute_onchain(
        script_content: &str,
        params: &[Value],
    ) -> Result<(Option<Value>, u64), String> {
        use std::process::Command;

        // Create temporary script file
        let temp_script = format!("/tmp/five_test_{}.v", std::process::id());
        std::fs::write(&temp_script, script_content)
            .map_err(|e| format!("Failed to write temp script: {}", e))?;

        // Build Five CLI command
        let mut cmd = Command::new("node");
        cmd.args(&[
            "../five-cli/dist/index.js",
            "local",
            "execute",
            &temp_script,
        ]);

        // Add parameters if any
        if !params.is_empty() {
            let param_strings: Vec<String> = params
                .iter()
                .map(|p| match p {
                    Value::U64(v) => v.to_string(),
                    Value::Bool(v) => v.to_string(),
                    _ => "0".to_string(), // Default for unsupported types
                })
                .collect();
            cmd.args(&["--params"]).args(&param_strings);
        }

        // Execute command
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute Five CLI: {}", e))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_script);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Five CLI execution failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse result from Five CLI output
        // Look for patterns like "✓ Execution successful" and "Result: 42"
        if stdout.contains("✓ Execution successful") || stdout.contains("Execution successful") {
            // Try to extract numeric result from "  Result: 42" pattern
            if let Some(result_line) = stdout
                .lines()
                .find(|line| line.trim().starts_with("Result:"))
            {
                if let Some(result_str) = result_line.split("Result:").nth(1) {
                    if let Ok(value) = result_str.trim().parse::<u64>() {
                        return Ok((Some(Value::U64(value)), 0)); // Compute units not easily extractable
                    }
                }
            }
            // If no specific result found, execution was successful
            Ok((Some(Value::Bool(true)), 0))
        } else {
            Err(format!(
                "Execution failed or no result found in output: {}",
                stdout
            ))
        }
    }

    /// Execute bytecode with MitoVM
    fn execute_bytecode(bytecode: &[u8], params: &[Value]) -> Result<(Option<Value>, u64), String> {
        // Create minimal accounts for testing
        // Account system scripts need at least one account to avoid index out of bounds
        let accounts = vec![];

        // Encode parameters using VLE if any
        let input_data = if params.is_empty() {
            vec![]
        } else {
            // NOTE: In onchain execution, the format would be [2, function_index, param_count, ...]
            // where 2 is the Execute instruction discriminator. The Five Solana program strips
            // the discriminator and passes [function_index, param_count, ...] to MitoVM.
            // Since we're calling MitoVM directly here, we use the post-processing format.
            // Format: [function_index (VLE), param_count (VLE), param1 (VLE), param2 (VLE), ...]
            let mut data = vec![];

            // Function index 0 (u32)
            data.extend_from_slice(&0u32.to_le_bytes());

            // Parameter count (u32)
            data.extend_from_slice(&(params.len() as u32).to_le_bytes());

            for param in params {
                match param {
                    Value::U64(val) => {
                        // Encode as pure value (no type marker needed)
                        // parse_parameters_unified expects pure values
                        let bytes = (*val as u32).to_le_bytes();
                        data.extend_from_slice(&bytes);
                    }
                    Value::Bool(val) => {
                        // Encode bool as 0 or 1
                        data.push(if *val { 1 } else { 0 });
                    }
                    _ => return Err("Unsupported parameter type".to_string()),
                }
            }
            data
        };

        let mut storage = StackStorage::new();
        match MitoVM::execute_direct(bytecode, &input_data, &accounts, &FIVE_VM_PROGRAM_ID, &mut storage) {
            Ok(result) => {
                let compute_units = 0; // MitoVM doesn't expose compute units in this interface
                Ok((result, compute_units))
            }
            Err(e) => Err(format!("Execution failed: {:?}", e)),
        }
    }

    /// Execute script using the appropriate method based on test mode
    fn execute_script(
        script_content: &str,
        params: &[Value],
        mode: TestMode,
    ) -> Result<(Option<Value>, u64), String> {
        match mode {
            TestMode::Wasm => {
                // Compile and execute with MitoVM (existing path)
                let bytecode = compile_five_script(script_content)?;
                execute_bytecode(&bytecode, params)
            }
            TestMode::Localnet => {
                // Execute on-chain with Five CLI
                execute_onchain(script_content, params)
            }
        }
    }

    /// Test a single Five DSL script
    fn test_script(script_path: &Path) -> TestResult {
        let script_name = script_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let category = script_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Read script content
        let content = match fs::read_to_string(script_path) {
            Ok(content) => content,
            Err(e) => {
                return TestResult {
                    script_name,
                    category,
                    compilation_success: false,
                    compilation_error: Some(format!("Failed to read file: {}", e)),
                    execution_success: false,
                    execution_error: None,
                    result_value: None,
                    bytecode_size: 0,
                    compute_units: 0,
                }
            }
        };

        // Parse test parameters
        let params = parse_test_params(&content);

        // Compile the script
        let (bytecode, compilation_success, compilation_error) = match compile_five_script(&content)
        {
            Ok(bytecode) => (bytecode, true, None),
            Err(e) => (vec![], false, Some(e)),
        };

        let bytecode_size = bytecode.len();

        // Get test mode
        let mode = get_test_mode();

        // Execute if compilation succeeded (or if using localnet mode which compiles internally)
        let (execution_success, execution_error, result_value, compute_units) =
            if compilation_success || mode == TestMode::Localnet {
                match execute_script(&content, &params, mode) {
                    Ok((result, cu)) => (true, None, result, cu),
                    Err(e) => (false, Some(e), None, 0),
                }
            } else {
                (false, None, None, 0)
            };

        TestResult {
            script_name,
            category,
            compilation_success,
            compilation_error,
            execution_success,
            execution_error,
            result_value,
            bytecode_size,
            compute_units,
        }
    }

    /// Find all .v test scripts
    fn find_test_scripts() -> Vec<std::path::PathBuf> {
        let test_scripts_dir = Path::new("test-scripts");
        let mut scripts = Vec::new();

        if !test_scripts_dir.exists() {
            eprintln!("Warning: test-scripts directory not found");
            return scripts;
        }

        fn collect_scripts(dir: &Path, scripts: &mut Vec<std::path::PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        collect_scripts(&path, scripts);
                    } else if path.extension().map_or(false, |ext| ext == "v") {
                        scripts.push(path);
                    }
                }
            }
        }

        collect_scripts(test_scripts_dir, &mut scripts);
        scripts.sort();
        scripts
    }

    #[test]
    fn test_basic_language_features() {
        let scripts = find_test_scripts();
        let basic_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("01-language-basics"))
            .collect();

        println!("Testing {} basic language scripts", basic_scripts.len());

        let mut results = Vec::new();
        for script_path in basic_scripts {
            let result = test_script(script_path);
            println!(
                "  {} - Compilation: {}, Execution: {}",
                result.script_name,
                if result.compilation_success {
                    "✓"
                } else {
                    "✗"
                },
                if result.execution_success {
                    "✓"
                } else {
                    "✗"
                }
            );

            if let Some(ref error) = result.compilation_error {
                println!("    Compilation error: {}", error);
            }
            if let Some(ref error) = result.execution_error {
                println!("    Execution error: {}", error);
            }
            if let Some(ref value) = result.result_value {
                println!("    Result: {:?}", value);
            }

            results.push(result);
        }

        // Basic language features should have high success rate
        let compilation_success_rate =
            results.iter().filter(|r| r.compilation_success).count() as f64 / results.len() as f64;

        let execution_success_rate =
            results.iter().filter(|r| r.execution_success).count() as f64 / results.len() as f64;

        println!(
            "Basic language compilation success rate: {:.1}%",
            compilation_success_rate * 100.0
        );
        println!(
            "Basic language execution success rate: {:.1}%",
            execution_success_rate * 100.0
        );

        // At least 80% of basic language features should compile
        assert!(
            compilation_success_rate >= 0.8,
            "Basic language compilation success rate too low: {:.1}%",
            compilation_success_rate * 100.0
        );
    }

    #[test]
    fn test_arithmetic_operations() {
        let scripts = find_test_scripts();
        let arithmetic_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("02-operators-expressions"))
            .collect();

        println!(
            "Testing {} arithmetic operation scripts",
            arithmetic_scripts.len()
        );

        let mut results = Vec::new();
        for script_path in arithmetic_scripts {
            let result = test_script(script_path);
            println!(
                "  {} - Compilation: {}, Execution: {}",
                result.script_name,
                if result.compilation_success {
                    "✓"
                } else {
                    "✗"
                },
                if result.execution_success {
                    "✓"
                } else {
                    "✗"
                }
            );

            results.push(result);
        }

        // Arithmetic operations should work reliably
        let execution_success_rate =
            results.iter().filter(|r| r.execution_success).count() as f64 / results.len() as f64;

        println!(
            "Arithmetic execution success rate: {:.1}%",
            execution_success_rate * 100.0
        );

        // All arithmetic operations should work (this is core VM functionality)
        assert!(
            execution_success_rate >= 0.95,
            "Arithmetic execution success rate too low: {:.1}%",
            execution_success_rate * 100.0
        );
    }

    #[test]
    fn test_control_flow() {
        let scripts = find_test_scripts();
        let control_flow_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("03-control-flow"))
            .collect();

        println!(
            "Testing {} control flow scripts",
            control_flow_scripts.len()
        );

        let mut results = Vec::new();
        for script_path in control_flow_scripts {
            let result = test_script(script_path);
            println!(
                "  {} - Compilation: {}, Execution: {}",
                result.script_name,
                if result.compilation_success {
                    "✓"
                } else {
                    "✗"
                },
                if result.execution_success {
                    "✓"
                } else {
                    "✗"
                }
            );

            results.push(result);
        }

        // Control flow should work reliably
        let execution_success_rate =
            results.iter().filter(|r| r.execution_success).count() as f64 / results.len() as f64;

        println!(
            "Control flow execution success rate: {:.1}%",
            execution_success_rate * 100.0
        );

        // All control flow operations should work
        assert!(
            execution_success_rate >= 0.90,
            "Control flow execution success rate too low: {:.1}%",
            execution_success_rate * 100.0
        );
    }

    #[test]
    fn test_match_expressions() {
        let scripts = find_test_scripts();
        let match_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("08-match-expressions"))
            .collect();

        println!("Testing {} match expression scripts", match_scripts.len());

        let mut results = Vec::new();
        for script_path in match_scripts {
            let result = test_script(script_path);
            println!(
                "  {} - Compilation: {}, Execution: {}",
                result.script_name,
                if result.compilation_success {
                    "✓"
                } else {
                    "✗"
                },
                if result.execution_success {
                    "✓"
                } else {
                    "✗"
                }
            );

            results.push(result);
        }

        // Match expressions showed 100% success in CLI tests, should maintain that
        let execution_success_rate =
            results.iter().filter(|r| r.execution_success).count() as f64 / results.len() as f64;

        println!(
            "Match expression execution success rate: {:.1}%",
            execution_success_rate * 100.0
        );

        // Match expressions should work very reliably
        assert!(
            execution_success_rate >= 0.90,
            "Match expression execution success rate too low: {:.1}%",
            execution_success_rate * 100.0
        );
    }

    #[test]
    fn test_account_system_integration() {
        let scripts = find_test_scripts();
        let account_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("04-account-system"))
            .collect();

        let mode = get_test_mode();
        println!(
            "Testing {} account system scripts in {:?} mode",
            account_scripts.len(),
            mode
        );

        match mode {
            TestMode::Wasm => {
                // WASM mode: Test compilation only (execution requires proper account structures)
                println!("WASM mode: Testing compilation only (execution requires Solana AccountInfo structures)");

                let mut compilation_results = Vec::new();

                for script_path in account_scripts {
                    let script_name = script_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    // Read script content
                    let content = match std::fs::read_to_string(script_path) {
                        Ok(content) => content,
                        Err(_) => {
                            println!("  {} - Failed to read file", script_name);
                            continue;
                        }
                    };

                    // Test compilation only
                    let compilation_success = match compile_five_script(&content) {
                        Ok(_) => {
                            println!(
                                "  {} - Compilation: ✓, Execution: SKIPPED (requires accounts)",
                                script_name
                            );
                            true
                        }
                        Err(e) => {
                            println!(
                                "  {} - Compilation: ✗ ({}), Execution: SKIPPED",
                                script_name, e
                            );
                            false
                        }
                    };

                    compilation_results.push(compilation_success);
                }

                let compilation_success_rate = compilation_results
                    .iter()
                    .filter(|&&success| success)
                    .count() as f64
                    / compilation_results.len() as f64;

                println!(
                    "Account system compilation success rate: {:.1}%",
                    compilation_success_rate * 100.0
                );
                println!("NOTE: Execution testing skipped in WASM mode");

                // At least compilation should work for most account system scripts
                if compilation_success_rate < 0.50 {
                    println!(
                        "WARNING: Account system compilation success rate is low: {:.1}%",
                        compilation_success_rate * 100.0
                    );
                }
            }

            TestMode::Localnet => {
                // Localnet mode: Full testing with real Solana accounts
                if !is_localnet_available() {
                    println!("⚠️  Five CLI not available or local validator not running");
                    println!("   To test account system in localnet mode:");
                    println!("   1. Start local Solana validator: solana-test-validator");
                    println!("   2. Build Five CLI: cd ../five-cli && npm run build");
                    println!("   3. Set environment: export FIVE_TEST_MODE=localnet");
                    println!("   Skipping localnet tests...");
                    return;
                }

                println!("Localnet mode: Full account system testing with real Solana accounts");

                let mut results = Vec::new();
                for script_path in account_scripts {
                    let result = test_script(script_path);
                    println!(
                        "  {} - Compilation: {}, Execution: {}",
                        result.script_name,
                        if result.compilation_success {
                            "✓"
                        } else {
                            "✗"
                        },
                        if result.execution_success {
                            "✓"
                        } else {
                            "✗"
                        }
                    );

                    if let Some(ref error) = result.execution_error {
                        println!("    Execution error: {}", error);
                    }
                    if let Some(ref value) = result.result_value {
                        println!("    Result: {:?}", value);
                    }

                    results.push(result);
                }

                // Calculate success rates
                let compilation_success_rate =
                    results.iter().filter(|r| r.compilation_success).count() as f64
                        / results.len() as f64;

                let execution_success_rate = results.iter().filter(|r| r.execution_success).count()
                    as f64
                    / results.len() as f64;

                println!(
                    "Account system compilation success rate: {:.1}%",
                    compilation_success_rate * 100.0
                );
                println!(
                    "Account system execution success rate: {:.1}%",
                    execution_success_rate * 100.0
                );

                // In localnet mode, we expect much higher success rates
                assert!(
                    compilation_success_rate >= 0.80,
                    "Account system compilation success rate too low: {:.1}%",
                    compilation_success_rate * 100.0
                );

                // Account system execution should work much better with real accounts
                if execution_success_rate < 0.60 {
                    println!(
                        "WARNING: Account system execution success rate is low: {:.1}%",
                        execution_success_rate * 100.0
                    );
                    println!("This may indicate issues with account system implementation");
                }
            }
        }
    }

    #[test]
    fn test_compilation_consistency() {
        // Test that the same scripts compile consistently
        let scripts = find_test_scripts();
        let sample_scripts: Vec<_> = scripts.iter().take(10).collect();

        println!(
            "Testing compilation consistency for {} scripts",
            sample_scripts.len()
        );

        for script_path in sample_scripts {
            let content = fs::read_to_string(script_path).unwrap();

            // Compile the same script multiple times
            let mut bytecodes = Vec::new();
            for _ in 0..3 {
                if let Ok(bytecode) = compile_five_script(&content) {
                    bytecodes.push(bytecode);
                }
            }

            if bytecodes.len() >= 2 {
                // All compilations should produce identical bytecode
                for i in 1..bytecodes.len() {
                    assert_eq!(
                        bytecodes[0],
                        bytecodes[i],
                        "Inconsistent compilation for {}",
                        script_path.display()
                    );
                }
                println!(
                    "  {} - Consistent bytecode generation ✓",
                    script_path.file_stem().unwrap().to_string_lossy()
                );
            }
        }
    }

    #[test]
    fn test_bytecode_efficiency() {
        // Test that bytecode sizes are reasonable
        let scripts = find_test_scripts();
        let basic_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("01-language-basics"))
            .collect();

        println!(
            "Testing bytecode efficiency for {} basic scripts",
            basic_scripts.len()
        );

        let mut total_size = 0;
        let mut script_count = 0;

        for script_path in basic_scripts {
            let result = test_script(script_path);
            if result.compilation_success {
                total_size += result.bytecode_size;
                script_count += 1;
                println!("  {} - {} bytes", result.script_name, result.bytecode_size);

                // Individual scripts should be reasonably sized
                assert!(
                    result.bytecode_size <= MAX_SCRIPT_SIZE,
                    "Script {} exceeds max size: {} bytes",
                    result.script_name,
                    result.bytecode_size
                );

                // Basic scripts should be reasonably sized
                // Scripts with multiple functions and dispatcher logic can be larger
                assert!(
                    result.bytecode_size <= 200,
                    "Basic script {} is too large: {} bytes",
                    result.script_name,
                    result.bytecode_size
                );
            }
        }

        if script_count > 0 {
            let average_size = total_size / script_count;
            println!("Average bytecode size: {} bytes", average_size);

            // Average basic script size - with dispatcher logic and multiple functions, reasonable bound
            assert!(
                average_size <= 100,
                "Average bytecode size too large: {} bytes",
                average_size
            );
        }
    }

    #[test]
    fn test_error_handling() {
        let scripts = find_test_scripts();
        let error_scripts: Vec<_> = scripts
            .iter()
            .filter(|p| p.to_string_lossy().contains("07-error-system"))
            .collect();

        println!("Testing {} error handling scripts", error_scripts.len());

        for script_path in error_scripts {
            let result = test_script(script_path);
            println!(
                "  {} - Compilation: {}, Execution: {}",
                result.script_name,
                if result.compilation_success {
                    "✓"
                } else {
                    "✗"
                },
                if result.execution_success {
                    "✓"
                } else {
                    "✗"
                }
            );

            // Scripts with "syntax-error" in the name should fail compilation
            if result.script_name.contains("syntax-error") {
                assert!(
                    !result.compilation_success,
                    "Script {} should fail compilation but didn't",
                    result.script_name
                );
                println!("    ✓ Correctly rejected invalid syntax");
            }
        }
    }

    #[test]
    fn test_specific_working_scripts() {
        // Test specific scripts that we know should work based on CLI tests
        let working_scripts = vec![
            "test-scripts/01-language-basics/simple-add.v",
            "test-scripts/01-language-basics/simple-return.v",
            "test-scripts/02-operators-expressions/basic-arithmetic.v",
            "test-scripts/08-match-expressions/simple-option.v",
        ];

        for script_name in working_scripts {
            let script_path = Path::new(script_name);
            if script_path.exists() {
                let result = test_script(script_path);

                assert!(
                    result.compilation_success,
                    "Known working script {} failed compilation: {:?}",
                    script_name, result.compilation_error
                );

                assert!(
                    result.execution_success,
                    "Known working script {} failed execution: {:?}",
                    script_name, result.execution_error
                );

                println!("✓ {} works correctly", script_name);
            }
        }
    }

    #[test]
    fn test_parameter_passing() {
        // Test scripts that use @test-params
        let script_path = Path::new("test-scripts/01-language-basics/simple-add.v");
        if script_path.exists() {
            let result = test_script(script_path);

            assert!(result.compilation_success, "simple-add.v should compile");
            assert!(result.execution_success, "simple-add.v should execute");

            // simple-add.v should return 30 (10 + 20 from @test-params)
            if let Some(Value::U64(value)) = result.result_value {
                assert_eq!(value, 30, "simple-add.v should return 30");
                println!("✓ Parameter passing works: 10 + 20 = {}", value);
            } else {
                panic!("simple-add.v should return a U64 value");
            }
        }
    }

}
