#!/usr/bin/env python3
"""
Script to add span: TokenSpan::default() to AST node constructions in test files.
"""
import re
import sys
from pathlib import Path

def add_span_to_ast_nodes(content):
    """Add span field to AST node struct initializations that are missing it."""
    # Pattern to match AstNode struct initializations without span field
    # This matches patterns like: AstNode::SomeVariant { field1: value1, field2: value2 }
    # and adds span: TokenSpan::default(), before the closing brace
    
    # First, add TokenSpan import if not present
    if 'use crate::tokenizer::TokenSpan' not in content and 'TokenSpan' in content:
        # Find the last use statement and add after it
        use_pattern = r'(use crate::[^;]+;)'
        matches = list(re.finditer(use_pattern, content))
        if matches:
            last_use = matches[-1]
            insert_pos = last_use.end()
            content = content[:insert_pos] + '\nuse crate::tokenizer::TokenSpan;' + content[insert_pos:]
    
    # Pattern for AstNode constructions that might be missing span
    # This is a simplified approach - we'll add span: TokenSpan::default(), before closing braces
    # in AstNode:: constructions
    
    # Match AstNode::Variant { ... } patterns
    pattern = r'(AstNode::\w+\s*\{[^}]+)(\})'
    
    def add_span_if_missing(match):
        content = match.group(1)
        closing = match.group(2)
        
        # Check if span is already present
        if 'span:' in content:
            return match.group(0)
        
        # Add span before closing brace
        # Handle trailing comma
        if content.rstrip().endswith(','):
            return f"{content} span: TokenSpan::default(),{closing}"
        else:
            return f"{content}, span: TokenSpan::default(){closing}"
    
    content = re.sub(pattern, add_span_if_missing, content)
    
    return content

def main():
    files_to_fix = [
        'src/bytecode_generator/function_dispatch.rs',
        'src/bytecode_generator/abi_generator.rs',
        'src/bytecode_generator/ast_generator/tests.rs',
        'src/bytecode_generator/module_merger.rs',
        'src/interface_registry.rs',
        'src/ast/conversions.rs',
        'src/lib.rs',
    ]
    
    base_path = Path('/Users/amberjackson/Documents/Development/five-org/five-dsl-compiler')
    
    for file_path in files_to_fix:
        full_path = base_path / file_path
        if not full_path.exists():
            print(f"Skipping {file_path} (not found)")
            continue
        
        print(f"Processing {file_path}...")
        content = full_path.read_text()
        new_content = add_span_to_ast_nodes(content)
        
        if content != new_content:
            full_path.write_text(new_content)
            print(f"  ✓ Updated {file_path}")
        else:
            print(f"  - No changes needed for {file_path}")
    
    print("\nDone!")

if __name__ == '__main__':
    main()
