// Benchmark comparing ValueAccessContext vs direct operations
// Run with: cargo bench --bench value_access_overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use five_protocol::ValueRef;

fn benchmark_direct_operations(c: &mut Criterion) {
    c.bench_function("direct_u64_ops", |b| {
        let values = vec![42u64, 123u64, 999u64, 1337u64, 9999u64];

        b.iter(|| {
            let mut sum = 0u64;
            for &value in &values {
                sum += black_box(value); // Prevent optimization
            }
            sum
        });
    });
}

fn benchmark_valueref_immediate(c: &mut Criterion) {
    c.bench_function("valueref_immediate_ops", |b| {
        let values = vec![
            ValueRef::U64(42),
            ValueRef::U64(123),
            ValueRef::U64(999),
            ValueRef::U64(1337),
            ValueRef::U64(9999),
        ];

        b.iter(|| {
            let mut sum = 0u64;
            for value_ref in &values {
                if let ValueRef::U64(v) = value_ref {
                    sum += black_box(*v);
                }
            }
            sum
        });
    });
}

fn benchmark_valueref_indirect(c: &mut Criterion) {
    c.bench_function("valueref_indirect_ops", |b| {
        // Simulate account data with u64 values
        let account_data = [
            42u64.to_le_bytes(),
            123u64.to_le_bytes(),
            999u64.to_le_bytes(),
            1337u64.to_le_bytes(),
            9999u64.to_le_bytes(),
        ]
        .concat();

        let values = vec![
            ValueRef::AccountRef(0, 0),  // Offset 0
            ValueRef::AccountRef(0, 8),  // Offset 8
            ValueRef::AccountRef(0, 16), // Offset 16
            ValueRef::AccountRef(0, 24), // Offset 24
            ValueRef::AccountRef(0, 32), // Offset 32
        ];

        b.iter(|| {
            let mut sum = 0u64;
            for value_ref in &values {
                // Simulate KISS direct reading from account data
                match value_ref {
                    ValueRef::AccountRef(_account_idx, offset) => {
                        let offset = *offset as usize;
                        if offset + 8 <= account_data.len() {
                            let bytes = &account_data[offset..offset + 8];
                            let value = u64::from_le_bytes([
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                bytes[6], bytes[7],
                            ]);
                            sum += black_box(value);
                        }
                    }
                    _ => continue,
                }
            }
            sum
        });
    });
}

fn benchmark_option_creation_direct(c: &mut Criterion) {
    c.bench_function("option_creation_direct", |b| {
        b.iter(|| {
            let values = vec![Some(42u64), Some(123u64), None, Some(999u64), None];

            // Process options
            let mut sum = 0u64;
            for opt in values {
                if let Some(v) = opt {
                    sum += black_box(v);
                }
            }
            sum
        });
    });
}

fn benchmark_option_creation_context(c: &mut Criterion) {
    c.bench_function("option_creation_kiss", |b| {
        b.iter(|| {
            // Create options using KISS AccountRef convention
            let mut options = Vec::new();

            // Some(42) - AccountRef with account 0
            options.push(ValueRef::AccountRef(0, 0));

            // Some(123) - AccountRef with account 0
            options.push(ValueRef::AccountRef(0, 8));

            // None - AccountRef with account 255 (special convention)
            options.push(ValueRef::AccountRef(255, 0));

            // Process options using KISS approach
            let mut sum = 0u64;
            for opt_ref in &options {
                match opt_ref {
                    ValueRef::AccountRef(255, _) => {
                        // None - skip
                        continue;
                    }
                    ValueRef::AccountRef(account_idx, _offset) if *account_idx < 254 => {
                        // Some value - for benchmark, use account_idx * 42 as mock value
                        sum += black_box(*account_idx as u64 * 42);
                    }
                    _ => continue,
                }
            }
            sum
        });
    });
}

criterion_group!(
    benches,
    benchmark_direct_operations,
    benchmark_valueref_immediate,
    benchmark_valueref_indirect,
    benchmark_option_creation_direct,
    benchmark_option_creation_context
);
criterion_main!(benches);
