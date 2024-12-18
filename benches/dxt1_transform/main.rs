use core::alloc::Layout;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dxt_lossless_transform::raw::*;
use safe_allocator_api::RawAlloc;

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

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    // Create a new allocation of 1024 bytes
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("DXT1 Transform Implementations");

    // Test different sizes
    // 16384 = 512x512
    for size in [16384].iter() {
        let input = generate_test_data(*size);
        let mut output = allocate_align_64(input.len());

        group.throughput(criterion::Throughput::Bytes(*size as u64));

        // Benchmark portable implementations
        let portable_impls = [
            (
                "portable64 (auto)",
                portable as unsafe fn(*const u8, *mut u8, usize),
            ),
            ("portable64 shift no-unroll", shift),
            ("portable64 shift unroll-2", shift_unroll_2),
            ("portable64 shift unroll-4", shift_unroll_4),
            ("portable64 shift unroll-8", shift_unroll_8),
        ];

        for (name, implementation) in portable_impls.iter() {
            group.bench_with_input(
                BenchmarkId::new(name.to_owned(), size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        implementation(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );
        }

        // Benchmark SSE2 implementations
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse2") {
                let sse2_impls = [
                    ("sse2 (auto)", sse2 as unsafe fn(*const u8, *mut u8, usize)),
                    ("sse2 punpckhqdq unroll-2", punpckhqdq_unroll_2),
                    ("sse2 punpckhqdq unroll-4", punpckhqdq_unroll_4),
                    ("sse2 punpckhqdq unroll-8", punpckhqdq_unroll_8),
                ];

                for (name, implementation) in sse2_impls.iter() {
                    group.bench_with_input(
                        BenchmarkId::new(name.to_owned(), size),
                        &size,
                        |b, &_size| {
                            b.iter(|| unsafe {
                                implementation(
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
