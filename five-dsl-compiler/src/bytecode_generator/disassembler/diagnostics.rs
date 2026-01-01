//! Diagnostic and error reporting utilities for bytecode analysis.

use crate::bytecode_generator::disassembler::disasm::disassemble;
use crate::bytecode_generator::disassembler::pretty::get_disassembly;

/// Produce a short diagnostic disassembly around `pos` with `ctx` bytes of context.
pub fn inspect_failure(bytes: &[u8], pos: usize, ctx: usize) -> String {
    let start = pos.saturating_sub(ctx);
    let end = std::cmp::min(bytes.len(), pos + ctx);
    let snippet = &bytes[start..end];

    let mut out = String::new();
    out.push_str(&format!("Bytecode failure near offset 0x{:04X}\n", pos));
    out.push_str(&format!(
        "Hex snippet [{}..{}): {:02X?}\n",
        start, end, snippet
    ));
    out.push_str("Disassembly (snippet):\n");
    for line in get_disassembly(snippet) {
        out.push_str("  ");
        out.push_str(&line);
        out.push('\n');
    }

    // Also detect truncated encodings
    let truncated_lines: Vec<String> = disassemble(snippet)
        .into_iter()
        .filter(|l| l.contains("<truncated>"))
        .collect();
    if !truncated_lines.is_empty() {
        out.push_str("\nDetected truncated instructions in the snippet:\n");
        for t in truncated_lines {
            out.push_str("  ");
            out.push_str(&t);
            out.push('\n');
        }
    }
    out
}
