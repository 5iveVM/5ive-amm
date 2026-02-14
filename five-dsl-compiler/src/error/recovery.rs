/// Error recovery and diagnostic utilities for Five DSL compiler
/// Helps provide better error messages and suggests fixes for common mistakes

/// Common Five DSL language elements for context-aware suggestions
pub struct LanguageContext {
    pub valid_types: &'static [&'static str],
    pub valid_keywords: &'static [&'static str],
    pub valid_constraints: &'static [&'static str],
}

impl Default for LanguageContext {
    fn default() -> Self {
        Self {
            valid_types: &[
                "u8", "u16", "u32", "u64",
                "bool",
                "string",
                "pubkey",
                "account",
            ],
            valid_keywords: &[
                "pub", "fn", "account", "let", "mut", "if", "else", "return",
                "require", "init", "use", "import",
            ],
            valid_constraints: &[
                "@mut", "@signer", "@init", "@pda",
            ],
        }
    }
}

/// Generate context-aware suggestions for parse errors
pub fn suggest_for_parse_error(expected: &str, found: &str) -> Option<Vec<String>> {
    let ctx = LanguageContext::default();

    // Type-related errors
    if found.contains("type '") {
        let type_name = found.trim_start_matches("type '").trim_end_matches("'");
        let mut suggestions = vec![];

        // Check if it's a known type
        if ctx.valid_types.contains(&type_name) {
            suggestions.push(format!(
                "`{}` is a valid type in Five DSL, but is being used in an invalid context",
                type_name
            ));
            suggestions.push("Check that you're using it in the right position in the statement".to_string());
        } else {
            // Suggest similar types
            if let Some(similar) = find_similar_type(type_name) {
                suggestions.push(format!("Did you mean `{}`?", similar));
            }
            suggestions.push(format!(
                "Valid types in Five DSL are: {}",
                ctx.valid_types.join(", ")
            ));
        }

        return Some(suggestions);
    }

    // Constraint token errors
    if found.contains("@") {
        let mut suggestions = vec![];
        suggestions.push("Constraints (@mut, @signer, @init) are only valid after account parameter names".to_string());
        suggestions.push(format!("Valid constraints are: {}", ctx.valid_constraints.join(", ")));
        return Some(suggestions);
    }

    // Missing separator errors
    if expected.contains(";") && !found.contains(";") {
        let suggestions = vec![
            "Add a semicolon `;` at the end of the statement".to_string(),
            "Most statements in Five require semicolon termination".to_string(),
        ];
        return Some(suggestions);
    }

    // Missing bracket/paren errors
    if expected.contains("}") {
        let suggestions = vec![
            "Add a closing brace `}` to complete the block".to_string(),
            "Check for missing closing braces in accounts, functions, or control structures".to_string(),
        ];
        return Some(suggestions);
    }

    None
}

/// Find a similar type name (for typo suggestions)
fn find_similar_type(input: &str) -> Option<&'static str> {
    let valid_types = &["u8", "u16", "u32", "u64", "bool", "string", "pubkey", "account"];

    // Simple fuzzy matching: look for types with >60% character overlap
    for &candidate in valid_types {
        let similarity = calculate_similarity(input, candidate);
        if similarity > 0.6 {
            return Some(candidate);
        }
    }

    // Check common misspellings
    match input {
        "pub" => Some("pubkey"),
        "int" | "integer" => Some("u64"),
        "float" | "double" | "f32" | "f64" => Some("u64"), // No floats, suggest integer
        "str" => Some("string"),
        "bool" | "boolean" => Some("bool"),
        _ => None,
    }
}

/// Calculate similarity between two strings (Levenshtein-based)
fn calculate_similarity(a: &str, b: &str) -> f32 {
    let len_a = a.len();
    let len_b = b.len();
    let max_len = len_a.max(len_b);

    if max_len == 0 {
        return 1.0;
    }

    // Simple Levenshtein calculation
    let mut prev = vec![0; len_b + 1];
    let mut curr = vec![0; len_b + 1];

    for i in 0..=len_a {
        curr[0] = i;
        for j in 0..=len_b {
            if i == 0 {
                curr[j] = j;
            } else {
                let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) { 0 } else { 1 };
                curr[j] = (prev[j] + 1)           // deletion
                    .min(curr[j - 1] + 1)         // insertion
                    .min(prev[j - 1] + cost);     // substitution
            }
        }
        prev = curr.clone();
    }

    let distance = curr[len_b];
    1.0 - (distance as f32 / max_len as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similar_type_pubkey() {
        assert_eq!(find_similar_type("pub"), Some("pubkey"));
    }

    #[test]
    fn test_similar_type_int() {
        assert_eq!(find_similar_type("int"), Some("u64"));
    }

    #[test]
    fn test_similarity_calculation() {
        let sim = calculate_similarity("u64", "u63");
        assert!(sim > 0.6); // Should be fairly similar
    }
}
