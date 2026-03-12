//! Document state management
//!
//! Tracks opened documents and their current content

use lsp_types::Url;
use std::collections::HashMap;

/// Represents an open document in the editor
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub language_id: String,
    pub version: i32,
    pub content: String,
}

impl Document {
    pub fn new(uri: Url, language_id: String, content: String) -> Self {
        Self {
            uri,
            language_id,
            version: 1,
            content,
        }
    }

    /// Update document content with new version
    pub fn update(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
    }

    /// Apply incremental change
    pub fn apply_change(&mut self, range: Option<lsp_types::Range>, text: String) {
        if let Some(range) = range {
            let lines: Vec<&str> = self.content.lines().collect();
            let mut new_content = String::new();

            let start_line = range.start.line as usize;
            let start_char = range.start.character as usize;
            let end_line = range.end.line as usize;
            let end_char = range.end.character as usize;

            // Add lines before the change
            for i in 0..start_line {
                if i > 0 {
                    new_content.push('\n');
                }
                new_content.push_str(lines[i]);
            }

            // Add part of start line before change
            if start_line < lines.len() {
                if start_line > 0 {
                    new_content.push('\n');
                }
                new_content.push_str(&lines[start_line][..start_char]);
            }

            // Add new text
            new_content.push_str(&text);

            // Add part of end line after change
            if end_line < lines.len() {
                if end_char < lines[end_line].len() {
                    new_content.push_str(&lines[end_line][end_char..]);
                }
                // Add remaining lines
                for i in (end_line + 1)..lines.len() {
                    new_content.push('\n');
                    new_content.push_str(lines[i]);
                }
            }

            self.content = new_content;
        } else {
            self.content = text;
        }
    }
}

/// Manages all open documents
#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<Url, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, uri: Url, language_id: String, content: String) -> Document {
        let doc = Document::new(uri.clone(), language_id, content);
        self.documents.insert(uri, doc.clone());
        doc
    }

    pub fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub fn get(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }

    pub fn get_mut(&mut self, uri: &Url) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }

    pub fn update_content(&mut self, uri: &Url, content: String, version: i32) {
        if let Some(doc) = self.get_mut(uri) {
            doc.update(content, version);
        }
    }

    pub fn apply_change(&mut self, uri: &Url, range: Option<lsp_types::Range>, text: String) {
        if let Some(doc) = self.get_mut(uri) {
            doc.apply_change(range, text);
        }
    }

    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Url, &Document)> {
        self.documents.iter()
    }
}
