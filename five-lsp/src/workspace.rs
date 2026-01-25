//! Workspace management for multi-file Five DSL projects

use lsp_types::Url;
use std::collections::HashSet;

/// Represents the workspace root(s)
#[derive(Debug, Clone)]
pub struct Workspace {
    roots: Vec<Url>,
    five_files: HashSet<Url>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            five_files: HashSet::new(),
        }
    }

    pub fn add_root(&mut self, root: Url) {
        if !self.roots.contains(&root) {
            self.roots.push(root);
        }
    }

    pub fn remove_root(&mut self, root: &Url) {
        self.roots.retain(|r| r != root);
    }

    pub fn roots(&self) -> &[Url] {
        &self.roots
    }

    pub fn register_file(&mut self, uri: Url) {
        if is_five_file(&uri) {
            self.five_files.insert(uri);
        }
    }

    pub fn unregister_file(&mut self, uri: &Url) {
        self.five_files.remove(uri);
    }

    pub fn five_files(&self) -> impl Iterator<Item = &Url> {
        self.five_files.iter()
    }

    pub fn is_five_file(&self, uri: &Url) -> bool {
        self.five_files.contains(uri)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

fn is_five_file(uri: &Url) -> bool {
    uri.path().ends_with(".v") || uri.path().ends_with(".five")
}
