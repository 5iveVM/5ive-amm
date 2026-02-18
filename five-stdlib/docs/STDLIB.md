# 5IVE Standard Library (Vendored v1)

This project was initialized with a vendored stdlib scaffold in `src/std`.

## Included modules

1. `src/std/prelude.v`
2. `src/std/builtins.v`
3. `src/std/interfaces/spl_token.v`
4. `src/std/interfaces/system_program.v`

## Import style (explicit)

```v
use std::builtins;
use std::interfaces::spl_token;
use std::interfaces::system_program;
```

## Migration path

Current mode is vendored stdlib.
Future mode may use external dependency distribution (planned command path: `5ive stdlib sync` / `5ive stdlib upgrade`).
