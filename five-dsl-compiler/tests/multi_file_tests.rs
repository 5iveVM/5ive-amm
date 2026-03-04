use five_dsl_compiler::compiler::{CompilationConfig, CompilationMode, DslCompiler};
use five_dsl_compiler::error::ErrorCode;
// Removed ModuleGraph and DslTokenizer from imports as they are not directly used in these tests
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

// Helper to create a temporary directory and write .v files into it for testing
fn create_test_project(
    files: HashMap<String, String>,
) -> Result<(TempDir, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let root_path = dir.path().to_path_buf();
    let src_dir = root_path.join("src");
    std::fs::create_dir_all(&src_dir)?; // Ensure src_dir exists

    let mut entry_point_path = PathBuf::new();

    for (name, content) in &files {
        let file_path = src_dir.join(name);
        // Create parent directories for nested files (e.g., src/utils/helpers.v)
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, content)?;
        println!("Created file: {:?}", file_path);
    }

    // Determine entry point path
    if files.contains_key("main.v") {
        entry_point_path = src_dir.join("main.v");
    } else if let Some(first_file) = files.keys().next() {
        // Fallback to any file if main.v is missing (for specific tests)
        entry_point_path = src_dir.join(first_file);
    }

    // Convert both to absolute paths if possible
    let root_path = std::fs::canonicalize(&root_path).unwrap_or(root_path);
    // Only canonicalize entry point if it exists (it should)
    if entry_point_path.exists() {
        entry_point_path = std::fs::canonicalize(&entry_point_path).unwrap_or(entry_point_path);
    }

    println!("Root path: {:?}", root_path);
    println!("Entry point: {:?}", entry_point_path);

    // Return TempDir to keep directory alive
    Ok((dir, root_path, entry_point_path))
}

#[test]
fn test_discover_modules_simple() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert("main.v".to_string(), "script main { use lib; }".to_string());
    files.insert("lib.v".to_string(), "script lib { }".to_string());

    let (_dir, root_path, entry_point_path) = create_test_project(files)?;

    let modules = DslCompiler::discover_modules(&entry_point_path)?;

    // Resolve paths for comparison
    let lib_path = root_path.join("src/lib.v").canonicalize()?;
    let main_path = root_path.join("src/main.v").canonicalize()?;

    // Use canonical paths for comparison to handle symlinks
    let modules_canonical: Vec<PathBuf> = modules
        .iter()
        .map(|m| {
            Path::new(m)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(m))
        })
        .collect();

    assert!(
        modules_canonical.contains(&lib_path),
        "Module list {:?} does not contain {:?}",
        modules_canonical,
        lib_path
    );
    assert!(
        modules_canonical.contains(&main_path),
        "Module list {:?} does not contain {:?}",
        modules_canonical,
        main_path
    );
    assert_eq!(modules.len(), 2);

    Ok(())
}

#[test]
fn test_discover_modules_nested_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use utils::helpers; }".to_string(),
    );
    files.insert(
        "utils/helpers.v".to_string(),
        "script helpers { }".to_string(),
    );

    let (_dir, root_path, entry_point_path) = create_test_project(files)?;

    let modules = DslCompiler::discover_modules(&entry_point_path)?;

    let helpers_path = root_path.join("src/utils/helpers.v").canonicalize()?;
    let main_path = root_path.join("src/main.v").canonicalize()?;

    let modules_canonical: Vec<PathBuf> = modules
        .iter()
        .map(|m| {
            Path::new(m)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(m))
        })
        .collect();

    assert!(
        modules_canonical.contains(&helpers_path),
        "Module list {:?} does not contain {:?}",
        modules_canonical,
        helpers_path
    );
    assert!(
        modules_canonical.contains(&main_path),
        "Module list {:?} does not contain {:?}",
        modules_canonical,
        main_path
    );
    assert_eq!(modules.len(), 2);

    Ok(())
}

#[test]
fn test_auto_discovery_simple_compilation() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use lib; pub fn do_stuff() { } }".to_string(),
    );
    files.insert(
        "lib.v".to_string(),
        "script lib { pub fn my_func() { } }".to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    assert!(bytecode.len() > 10); // Should contain some instructions

    Ok(())
}

#[test]
fn test_auto_discovery_circular_dependency_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert("a.v".to_string(), "script a { use b; }".to_string());
    files.insert("b.v".to_string(), "script b { use a; }".to_string());

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Circular dependency"));

    Ok(())
}

#[test]
fn test_auto_discovery_missing_module_error() -> Result<(), Box<dyn std::error::Error>> {
    // We need to create a project where main.v refers to a non-existent module
    // But create_test_project only writes files we give it.
    // So we provide main.v, but NOT the module it uses.
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use non_existent; }".to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);

    assert!(result.is_err(), "Expected error for missing module");
    let err = result.unwrap_err();
    assert_eq!(
        err.code,
        ErrorCode::FILE_NOT_FOUND,
        "Expected FILE_NOT_FOUND error code, got {:?}",
        err.code
    );

    Ok(())
}

#[test]
fn test_compile_modules_explicit_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use lib; pub fn test_main() { } }".to_string(),
    );
    files.insert(
        "lib.v".to_string(),
        "script lib { pub fn test_lib() { } }".to_string(),
    );

    let (_dir, root_path, _entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let main_path = root_path.join("src/main.v").to_string_lossy().to_string();
    let lib_path = root_path.join("src/lib.v").to_string_lossy().to_string();

    let module_files = vec![main_path.clone(), lib_path.clone()];

    let bytecode = DslCompiler::compile_modules(module_files, &main_path, &config)?;
    assert!(!bytecode.is_empty());
    assert!(bytecode.len() > 10);

    Ok(())
}

#[test]
fn test_call_external_generation_via_auto_discovery() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use \"11111111111111111111111111111111\"; pub fn main_func() { } }"
            .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());

    Ok(())
}

#[test]
fn test_bundled_stdlib_named_import_auto_discovery_currently_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use std::builtins::{now_seconds}; pub fn run() -> u64 { return now_seconds(); } }"
            .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing).with_module_namespaces(false);

    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_bundled_stdlib_module_qualified_auto_discovery_currently_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use std::builtins; pub fn run() -> u64 { return builtins::now_seconds(); } }"
            .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_bundled_stdlib_full_qualified_auto_discovery_currently_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use std::builtins; pub fn run() -> u64 { return std::builtins::now_seconds(); } }"
            .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);

    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_ambiguous_unqualified_call_fails() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { use alpha; use beta; pub fn run() -> u64 { return same(); } }".to_string(),
    );
    files.insert(
        "alpha.v".to_string(),
        "script alpha { pub fn same() -> u64 { return 1; } }".to_string(),
    );
    files.insert(
        "beta.v".to_string(),
        "script beta { pub fn same() -> u64 { return 2; } }".to_string(),
    );

    let (_dir, root_path, _entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let main_path = root_path.join("src/main.v").to_string_lossy().to_string();
    let alpha_path = root_path.join("src/alpha.v").to_string_lossy().to_string();
    let beta_path = root_path.join("src/beta.v").to_string_lossy().to_string();

    let result = DslCompiler::compile_modules(
        vec![main_path.clone(), alpha_path, beta_path],
        &main_path,
        &config,
    );
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_bundled_stdlib_extended_builtin_wrappers_currently_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use std::builtins;
            pub fn run(a: u64, b: u64, c: u64) {
                builtins::log_message(a);
                builtins::log_words(a, b, c, 0, 0);
                builtins::memory_copy(a, b, c);
                builtins::memory_compare(a, b, c);
                builtins::return_data_set(a);
                builtins::remaining_cu();
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_bundled_stdlib_spl_token_extended_interface_currently_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use std::interfaces::spl_token;
            pub fn run(
                source: Account,
                destination: Account,
                mint: Account,
                authority: Account,
                delegate: Account
            ) {
                spl_token::transfer(source, destination, authority, 1);
                spl_token::approve(source, delegate, authority, 1);
                spl_token::revoke(source, authority);
                spl_token::mint_to(mint, destination, authority, 1);
                spl_token::burn(source, mint, authority, 1);
                spl_token::close_account(source, destination, authority);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_bundled_stdlib_system_program_extended_interface_compile(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use std::interfaces::system_program;
            pub fn run(
                payer: Account,
                new_account: Account,
                base: Account,
                owner: Account
            ) {
                system_program::SystemProgram::transfer(payer, new_account, 1);
                system_program::SystemProgram::assign(new_account, owner);
                system_program::SystemProgram::create_account(payer, new_account, 1, 8, owner);
                system_program::SystemProgram::create_account_with_seed(payer, new_account, base, 0, 1, 8, owner);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_bundled_stdlib_legacy_object_style_interface_call_fails() {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use std::interfaces::spl_token;
            pub fn run(
                source: Account,
                destination: Account,
                authority: Account
            ) {
                SPLToken.transfer(source, destination, authority, 1);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files).unwrap();
    let config = CompilationConfig::new(CompilationMode::Testing);
    let err = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)
        .expect_err("legacy object-style interface calls must fail");
    let err_text = err.to_string();
    assert!(
        err_text.contains("Undefined")
            || err_text.contains("undefined")
            || err_text.contains("Cannot find value")
            || err_text.contains("cannot find value")
            || err_text.contains("Constraint")
            || err_text.contains("constraint"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn test_local_module_interface_symbol_import_and_call_compile(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use remote::RemoteSink;
            pub fn run(target: Account) {
                RemoteSink::submit(target, \"vault\");
            }
        }"
        .to_string(),
    );
    files.insert(
        "remote.v".to_string(),
        "script remote {
            interface RemoteSink @program(\"11111111111111111111111111111111\") @serializer(raw) {
                submit @discriminator_bytes([]) (target: Account, label: string<32>);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_local_module_namespace_import_with_explicit_interface_path_compile(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use remote;
            pub fn run(target: Account) {
                remote::RemoteSink::submit(target, \"vault\");
            }
        }"
        .to_string(),
    );
    files.insert(
        "remote.v".to_string(),
        "script remote {
            interface RemoteSink @program(\"11111111111111111111111111111111\") @serializer(raw) {
                submit @discriminator_bytes([]) (target: Account, label: string<32>);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_local_module_value_symbol_import_and_call_compile(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use remote::submit;
            pub fn run() {
                submit();
            }
        }"
        .to_string(),
    );
    files.insert(
        "remote.v".to_string(),
        "script remote {
            pub fn submit() { }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_local_module_brace_type_and_value_import_compile(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use remote::{Pool, submit};
            pub fn run(pool: Pool @mut) {
                submit();
                pool.balance = 1;
            }
        }"
        .to_string(),
    );
    files.insert(
        "remote.v".to_string(),
        "script remote {
            account Pool {
                balance: u64;
            }

            pub fn submit() { }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_duplicate_imported_type_symbol_in_same_namespace_fails() {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use left::Pool;
            use right::Pool;
        }"
        .to_string(),
    );
    files.insert(
        "left.v".to_string(),
        "script left {
            account Pool { balance: u64; }
        }"
        .to_string(),
    );
    files.insert(
        "right.v".to_string(),
        "script right {
            account Pool { amount: u64; }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files).unwrap();
    let config = CompilationConfig::new(CompilationMode::Testing);
    let err = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)
        .expect_err("duplicate type imports should fail");
    let err_text = err.to_string();
    assert!(err_text.contains("duplicate imported type symbol `Pool`"));
}

#[test]
fn test_duplicate_imported_value_symbol_in_same_namespace_fails() {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use left::submit;
            use right::submit;
        }"
        .to_string(),
    );
    files.insert(
        "left.v".to_string(),
        "script left {
            pub fn submit() { }
        }"
        .to_string(),
    );
    files.insert(
        "right.v".to_string(),
        "script right {
            pub fn submit() { }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files).unwrap();
    let config = CompilationConfig::new(CompilationMode::Testing);
    let err = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)
        .expect_err("duplicate value imports should fail");
    let err_text = err.to_string();
    assert!(err_text.contains("duplicate imported value symbol `submit`"));
}

#[test]
fn test_module_and_type_import_can_coexist_across_namespaces(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use remote;
            use remote::RemoteSink;
            pub fn run(target: Account) {
                remote::RemoteSink::submit(target, \"vault\");
                RemoteSink::submit(target, \"vault\");
            }
        }"
        .to_string(),
    );
    files.insert(
        "remote.v".to_string(),
        "script remote {
            interface RemoteSink @program(\"11111111111111111111111111111111\") @serializer(raw) {
                submit @discriminator_bytes([]) (target: Account, label: string<32>);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_old_explicit_interface_import_syntax_rejected() {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use remote::{interface RemoteSink};
        }"
        .to_string(),
    );
    files.insert("remote.v".to_string(), "script remote { }".to_string());

    let (_dir, _root_path, entry_point_path) = create_test_project(files).unwrap();
    let config = CompilationConfig::new(CompilationMode::Testing);
    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
}

#[test]
fn test_bundled_stdlib_interface_symbol_import_compile(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main {
            use std::interfaces::spl_token::SPLToken;
            pub fn run(mint: Account, destination: Account, authority: Account @signer) {
                SPLToken::mint_to(mint, destination, authority, 1);
            }
        }"
        .to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let bytecode = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config)?;
    assert!(!bytecode.is_empty());
    Ok(())
}

#[test]
fn test_builtins_unqualified_call_without_import_fails() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { pub fn run() -> u64 { return now_seconds(); } }".to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_builtins_module_qualified_call_without_import_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { pub fn run() -> u64 { return builtins::now_seconds(); } }".to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_builtins_fully_qualified_call_without_import_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert(
        "main.v".to_string(),
        "script main { pub fn run() -> u64 { return std::builtins::now_seconds(); } }".to_string(),
    );

    let (_dir, _root_path, entry_point_path) = create_test_project(files)?;
    let config = CompilationConfig::new(CompilationMode::Testing);
    let result = DslCompiler::compile_with_auto_discovery(&entry_point_path, &config);
    assert!(result.is_err());
    Ok(())
}
