//! Template fixture execution belongs to BPF runtime tests.
//! Keep this file for lightweight fixture-shape checks only.

use std::path::PathBuf;

#[test]
fn template_fixture_directories_exist() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    assert!(repo_root.join("five-templates").exists());
}
