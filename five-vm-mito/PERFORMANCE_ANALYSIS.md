# Multi-Precision Arithmetic Performance Analysis

This note summarizes benchmark observations for u128/u256 arithmetic in MitoVM. It is informational only and may be stale as implementations evolve.

## Summary
- u128 add/mul is significantly slower than native Rust u128 on the test machine.
- u256 add/mul provides capabilities without native equivalents but still carries overhead.
- Binary size impact is modest.

## Observations
- u128 implementations use widening operations internally, which reduces the value of a custom path.
- u256 multiplication logic is complex relative to the measured benefit.
- Some operations truncate results, which may surprise users who expect full-width outputs.

## Suggested Direction
- Prefer native u128 where available; keep multi-precision as a fallback if needed.
- Keep u256 support focused and measurable.
- Add performance tracking if these paths are relied on in production.

## Environment (at time of measurement)
- Rust 1.75+ with -O
- Apple Silicon M1

