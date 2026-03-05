# @5ive/std

Canonical standard library source package for the compiler-bundled 5IVE stdlib.

Current consumption mode:
1. `five-dsl-compiler` resolves `std::...` imports through alias-driven dependency entries that map to this package.

Future consumption mode:
1. direct dependency/linking once package/dependency resolution is ready.
