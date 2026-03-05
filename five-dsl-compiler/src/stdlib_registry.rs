use std::path::{Path, PathBuf};

pub const STDLIB_PACKAGE_NAME: &str = "@5ive/std";
pub const STDLIB_DEFAULT_ALIAS: &str = "std";

pub fn bundled_stdlib_virtual_path(module_path: &str) -> PathBuf {
    let rel = module_path.replace("::", "/");
    PathBuf::from(format!("<bundled-stdlib>/{}.v", rel))
}

pub fn bundled_stdlib_module_path(stdlib_root: &Path, module_tail: &str) -> PathBuf {
    stdlib_root
        .join("std")
        .join(module_tail.replace("::", "/"))
        .with_extension("v")
}

pub fn find_bundled_stdlib_root(project_root: &Path) -> Option<PathBuf> {
    if let Ok(override_path) = std::env::var("FIVE_STDLIB_ROOT") {
        let root = PathBuf::from(override_path);
        if root.exists() {
            return Some(root);
        }
    }

    let mut cursor = Some(project_root.to_path_buf());
    while let Some(dir) = cursor {
        let candidates = [
            dir.join("five-cli").join("dist").join("assets").join("stdlib"),
            dir.join("five-cli").join("assets").join("stdlib"),
            dir.join("five-stdlib"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return Some(candidate);
            }
        }
        cursor = dir.parent().map(|p| p.to_path_buf());
    }

    None
}
