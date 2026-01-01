# Five DSL Syntax Bible (Updated)

This document summarizes the current Five DSL surface syntax as reflected by the canonical examples in `five-cli/test-scripts` and the Tree‑sitter grammar. It is the single place to quickly verify what the parser should accept and what the examples demonstrate.

## File Structure
- Top‑level items are declared directly; no `script { ... }` wrapper is required.
- Supported top‑level items:
  - Account definitions: `account Name { field: type; ... }`
  - Global state variables: `mut var_name: type;`
  - Init block: `init { ... }`
  - Functions: `pub? name(param_list?) ->? return_type? { ... }`

See: `01-language-basics/*.v`, `04-account-system/*.v`, `parameter_test_examples.v`

## Comments
- Line comments: `// ...` (used for inline docs and test hints)
- Test parameters convention: `// @test-params 10 20`

See: `01-language-basics/simple-add.v`

## Accounts and State
- Define accounts with fields:
  ```
  account StateAccount {
      count: u64;
  }
  ```
- Global mutable state:
  ```
  mut balance: u64;
  mut value: u64;
  ```

See: `04-account-system/account-definition.five`, `parameter_test_examples.v`

## Init Block
- Runs at initialization to set defaults:
  ```
  init {
      balance = 0;
      value = 0;
  }
  ```

See: `parameter_test_examples.v`

## Functions
- Declaration:
  ```
  pub add(a: u64, b: u64) -> u64 {
      return a + b;
  }
  ```
- Public modifier `pub` is optional.
- Return type is optional for procedures.

See: `01-language-basics/simple-add.v`, `parameter_test_examples.v`

## Parameters and Constraints
- Parameter syntax: `name: type` with optional constraints appended:
  - `@signer`, `@mut`, `@init`
  ```
  transfer(to: StateAccount @mut, authority: Pubkey @signer) {
      // ...
  }
  ```

See: `04-account-system/*`

### Examples
- Signer and mutable account parameter:
  - File: `five-cli/test-scripts/04-account-system/signer-constraint.v`
  - Pattern:
    ```
    update(owner: StateAccount @signer @mut, amount: u64) {
        require(amount > 0);
        owner.count = owner.count + amount;
    }
    ```

- Init account from function parameter:
  - File: `five-cli/test-scripts/04-account-system/init-constraint.v`
  - Pattern:
    ```
    create(new_state: StateAccount @init, seed: u64) {
        require(seed > 0);
        new_state.count = seed;
    }
    ```

## Statements
- Assignment: `x = expr;`
- Return: `return expr?;`
- Require (assert contract): `require(condition);`
- Expression statement: `expr;`

See: `01-language-basics/simple-return.v`, `parameter_test_examples.v`

### Additional examples
- require with comparison:
  - File: `five-cli/test-scripts/parameter_test_examples.v`
  - Snippet: `require(amount > 0);`
- simple return value:
  - File: `five-cli/test-scripts/01-language-basics/simple-return.v`
  - Snippet: `return 42;`

## Expressions and Operators
- Arithmetic: `+ - * / %`
- Comparisons: `== != < <= > >=`
- Logical: `&& || !`
- Field access: `obj.field`
- Function calls: `name(arg1, arg2)`
- Parentheses for grouping: `(expr)`
- Bitwise and shift operators are NOT part of the Five DSL.

See: `02-operators-expressions/*`

Notes:
- Bitwise operators (`|`, `^`, `&`) and shifts (`<<`, `>>`) are not part of the DSL and should be rejected by the compiler and grammar.

## Types
- Primitives: `u8, u16, u32, u64, i8, i16, i32, i64, bool, pubkey, string`
- Custom (account) types by identifier.

## Testing Conventions
- Use `// @test-params ...` at top of functions or files to embed test vectors.
- Test fixtures and examples live under `five-cli/test-scripts/**` and are used to validate parsing, compilation, and VM execution paths.

## Notes
- Legacy `script Name { ... }` wrapper is tolerated by the parser for backward compatibility but is not required in current examples.
- The Tree‑sitter grammar reflects this by allowing top‑level items directly and an optional legacy wrapper.
