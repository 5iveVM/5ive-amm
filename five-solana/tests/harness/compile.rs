use std::{
    fs,
    io,
    path::{Path, PathBuf},
};
use five_dsl_compiler::DslCompiler;

/// Load committed bytecode artifacts for runtime tests.
///
/// Optional compile bridge can be enabled later; currently this function
/// intentionally prefers deterministic checked-in artifacts.
pub fn load_or_compile_bytecode(script_path: &Path) -> io::Result<Vec<u8>> {
    if script_path.exists() {
        if is_five_source(script_path) {
            return compile_v_source(script_path);
        }
        return fs::read(script_path);
    }

    if std::env::var("FIVE_RUNTIME_COMPILE").ok().as_deref() == Some("1") {
        return Err(io::Error::other(format!(
            "compile bridge is not enabled yet for {} - provide a committed .bin artifact",
            script_path.display()
        )));
    }

    let candidates = fallback_candidates(script_path);
    for candidate in candidates {
        if candidate.exists() {
            if is_five_source(&candidate) {
                return compile_v_source(&candidate);
            }
            return fs::read(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("bytecode artifact not found for {}", script_path.display()),
    ))
}

fn is_five_source(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("v"))
        .unwrap_or(false)
}

fn compile_v_source(path: &Path) -> io::Result<Vec<u8>> {
    let source = fs::read_to_string(path)?;
    DslCompiler::compile_dsl(&source)
        .map_err(|e| io::Error::other(format!("failed compiling {}: {}", path.display(), e)))
}

fn fallback_candidates(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Some(stem) = path.file_stem() {
        let mut sibling_bin = path.to_path_buf();
        sibling_bin.set_file_name(format!("{}.bin", stem.to_string_lossy()));
        out.push(sibling_bin);

        if let Some(parent) = path.parent() {
            out.push(parent.join("build").join(format!("{}.bin", stem.to_string_lossy())));
            out.push(parent.join("src").join(format!("{}.bin", stem.to_string_lossy())));
        }
    }

    out
}

/// Optionally write generated Five source files for inspection or manual runs.
///
/// Disabled by default. Set `FIVE_WRITE_GENERATED_V=1` (or `true`/`yes`) to enable.
/// Optional override: `FIVE_GENERATED_V_DIR=/custom/dir`.
pub fn maybe_write_generated_v(repo_root: &Path, relative_path: &str, source: &str) {
    if !write_generated_v_enabled() {
        return;
    }

    let output_path = generated_v_output_path(repo_root, relative_path);
    if let Some(parent) = output_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "Warning: could not create generated source directory {}: {}",
                parent.display(),
                e
            );
            return;
        }
    }

    match fs::write(&output_path, source) {
        Ok(()) => println!("Generated Five source written to: {}", output_path.display()),
        Err(e) => eprintln!(
            "Warning: could not write generated source to {}: {}",
            output_path.display(),
            e
        ),
    }
}

fn write_generated_v_enabled() -> bool {
    matches!(
        std::env::var("FIVE_WRITE_GENERATED_V"),
        Ok(value) if matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes")
    )
}

fn generated_v_output_path(repo_root: &Path, relative_path: &str) -> PathBuf {
    let requested = PathBuf::from(relative_path);
    if requested.is_absolute() {
        return requested;
    }

    if let Ok(base_dir) = std::env::var("FIVE_GENERATED_V_DIR") {
        return PathBuf::from(base_dir).join(relative_path);
    }

    repo_root.join("five-templates").join(relative_path)
}
