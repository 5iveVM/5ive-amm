use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

/// Load committed bytecode artifacts for runtime tests.
///
/// Optional compile bridge can be enabled later; currently this function
/// intentionally prefers deterministic checked-in artifacts.
pub fn load_or_compile_bytecode(script_path: &Path) -> io::Result<Vec<u8>> {
    if script_path.exists() {
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
            return fs::read(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("bytecode artifact not found for {}", script_path.display()),
    ))
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
