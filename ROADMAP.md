# Five VM & DSL Compiler Roadmap

## Phase 1: Core Compiler Features (Completed)
- [x] Basic parsing and tokenization
- [x] Type checking infrastructure
- [x] Basic bytecode generation
- [x] Function calls and parameter passing
- [x] Initial account system integration

## Phase 2: Compiler Robustness & Error Handling (Completed)
- [x] Remove hardcoded field offset heuristics
- [x] Implement proper `AccountSystem` integration for field offsets
- [x] Fix invalid variable index issues
- [x] Implement enhanced error codes (E0000-E3000)
- [x] Remove silent fallbacks in bytecode generation (Critical)
    - [x] Parameter encoding `unwrap_or(0)`
    - [x] Type size calculation fallbacks
    - [x] Account space default `1024`
    - [x] Array size fallbacks
    - [x] Field type defaults
    - [x] External function offset fallbacks

## Phase 3: Advanced Features (Current)
- [ ] Cross-Program Invocation (CPI)
    - [ ] `invoke` and `invoke_signed` parsing
    - [ ] Bytecode generation for CPI
    - [ ] VM implementation of CPI opcodes
- [ ] Program Derived Addresses (PDA)
    - [ ] `derive_pda` and `find_pda` built-ins
    - [ ] Seed generation and handling
- [ ] Advanced Account Constraints
    - [ ] `@init` constraint implementation
    - [ ] `@seeds` and `@bump` support
    - [ ] Constraint validation optimization

## Phase 4: Production Readiness
- [ ] Comprehensive test suite expansion
- [ ] Documentation generation
- [ ] Performance benchmarking and optimization
- [ ] Security audit fixes

## Phase 5: Ecosystem Tools
- [ ] Language Server Protocol (LSP) implementation
- [ ] VS Code extension
- [ ] Advanced CLI tools (deploy, test runner enhancements)
