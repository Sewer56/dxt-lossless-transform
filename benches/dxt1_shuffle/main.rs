use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use dxt_lossless_transform::raw::dxt1_block::*;
#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

// Helper to generate test data with predictable patterns
fn generate_test_data(num_blocks: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(num_blocks * 8);

    for i in 0..num_blocks {
        // Colors: Sequential bytes 1-64
        data.extend_from_slice(&[
            (1 + i * 4) as u8,
            (2 + i * 4) as u8,
            (3 + i * 4) as u8,
            (4 + i * 4) as u8,
        ]);

        // Indices: Sequential bytes 128-191
        data.extend_from_slice(&[
            (128 + i * 4) as u8,
            (129 + i * 4) as u8,
            (130 + i * 4) as u8,
            (131 + i * 4) as u8,
        ]);
    }
    data
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("DXT1 Baseline Implementations");

    // Test different sizes from small cache-friendly to larger sizes
    for size in [32, 256, 1024, 4096, 16384].iter() {
        let input = generate_test_data(*size);
        let mut output = vec![0u8; input.len()];

        group.throughput(criterion::Throughput::Bytes(*size as u64));

        // Benchmark portable version
        group.bench_with_input(BenchmarkId::new("portable", size), &size, |b, &_size| {
            b.iter(|| unsafe {
                transform_64bit_portable(
                    black_box(input.as_ptr()),
                    black_box(output.as_mut_ptr()),
                    black_box(input.len()),
                )
            });
        });

        // Benchmark SSE2 version
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                group.bench_with_input(
                    BenchmarkId::new("sse2_unroll16", size),
                    &size,
                    |b, &_size| {
                        b.iter(|| unsafe {
                            transform_64bit_sse2(
                                black_box(input.as_ptr()),
                                black_box(output.as_mut_ptr()),
                                black_box(input.len()),
                            )
                        });
                    },
                );
            }
        }
    }

    group.finish();

    #[cfg(not(feature = "pgo"))]
    {
        // Benchmarks excluded from PGO run.
    }
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(benches);
