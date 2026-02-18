# 5IVE Standard Library (Bundled v1)

The compiler provides stdlib modules from a bundled source registry.
Local `src/std` files are ignored in bundled mode.

## Included modules

1. `std::prelude`
2. `std::builtins`
3. `std::interfaces::spl_token`
4. `std::interfaces::system_program`

## Import style (explicit)

```v
use std::builtins::{now_seconds};
use std::interfaces::spl_token;
use std::interfaces::system_program;
```

Also supported:

```v
use std::builtins;
let now = builtins::now_seconds();
```

## Migration path

Current mode is bundled/inlined stdlib.
Future mode may support external dependency linkage.
