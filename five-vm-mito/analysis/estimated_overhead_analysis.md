# ValueAccessContext Overhead Notes

Measured results are from criterion benchmarks on 2025-08-30. Estimates below are rough and should be treated as placeholders until confirmed.

## Measured Results

Before KISS optimization (ValueAccessContext):
```
Operation Type           Direct Time    Context Time    Overhead
Simple u64 read          2.07 ns       2.67 ns         +29%
Account data read        2.07 ns       22.66 ns        +994%
Option<u64> creation     0.88 ns       30.33 ns        +3343%
```

After KISS optimization (AccountRef convention):
```
Operation Type           Direct Time    KISS Time       Overhead
Simple u64 read          2.06 ns       2.67 ns         +29%
Account data read        2.06 ns       5.04 ns         +145%
Option<u64> creation     0.87 ns       21.25 ns        +2341%
```

## Estimated Scaling (Unverified)

Medium scale (10-100 ops):
```
Operation Type          Direct     Context    Estimated Overhead
Array processing        ~500 cycles ~450 cycles ~-10%
JSON parsing            ~2000 cycles ~1200 cycles ~-40%
Complex data structures ~5000 cycles ~2500 cycles ~-50%
```

Large scale (100+ ops):
```
Operation Type          Direct      Context     Estimated Overhead
Bulk data processing    ~50,000 cy   ~15,000 cy  ~-70%
Memory allocation costs ~100,000 cy  ~1,000 cy   ~-99%
Cache misses           ~200,000 cy   ~5,000 cy   ~-97.5%
```

## Break-Even Estimates (Unverified)
- 1-2 simple values: direct likely faster
- 3-5 values: rough break-even
- 6+ values: context likely faster

## Notes
- Stack usage: context can be more predictable in BPF.
- Compute unit estimates should be validated with real benchmarks.

