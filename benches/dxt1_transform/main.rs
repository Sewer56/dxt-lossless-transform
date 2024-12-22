use core::alloc::Layout;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dxt_lossless_transform::raw::*;
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

// Helper to generate test data with predictable patterns
fn generate_test_data(num_blocks: usize) -> RawAlloc {
    let mut data = allocate_align_64(num_blocks * 8);
    let data_ptr = data.as_mut_ptr();

    for block_idx in 0..num_blocks {
        // Colors: Sequential bytes 1-64 (ensuring no overlap with indices)
        unsafe {
            *data_ptr.add(block_idx * 4) = (1 + block_idx * 4) as u8;
            *data_ptr.add(block_idx * 4 + 1) = (2 + block_idx * 4) as u8;
            *data_ptr.add(block_idx * 4 + 2) = (3 + block_idx * 4) as u8;
            *data_ptr.add(block_idx * 4 + 3) = (4 + block_idx * 4) as u8;
        }

        // Indices: Sequential bytes 128-191 (well separated from colors)
        unsafe {
            *data_ptr.add(block_idx * 4 + 4) = (128 + block_idx * 4) as u8;
            *data_ptr.add(block_idx * 4 + 5) = (129 + block_idx * 4) as u8;
            *data_ptr.add(block_idx * 4 + 6) = (130 + block_idx * 4) as u8;
            *data_ptr.add(block_idx * 4 + 7) = (131 + block_idx * 4) as u8;
        }
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
    let size = 16384;
    let input = generate_test_data(size);
    let mut output = allocate_align_64(input.len());

    group.throughput(criterion::Throughput::Bytes(size as u64));

    // Benchmark portable implementations
    let portable_impls = [
        (
            "portable64 (auto)",
            portable as unsafe fn(*const u8, *mut u8, usize),
        ),
        ("portable64 shift no-unroll", shift),
        ("portable64 shift_with_count no-unroll", shift_with_count),
        ("portable32 no-unroll", u32),
        ("portable64 shift unroll-2", shift_unroll_2),
        ("portable64 shift_with_count unroll-2", shift_unroll_2),
        ("portable32 unroll-2", u32_unroll_2),
        ("portable64 shift unroll-4", shift_unroll_4),
        ("portable64 shift_with_count unroll-4", shift_unroll_4),
        ("portable32 unroll-4", u32_unroll_4),
        ("portable64 shift unroll-8", shift_unroll_8),
        ("portable64 shift_with_count unroll-8", shift_unroll_8),
        ("portable32 unroll-8", u32_unroll_8),
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
        // Note: The functions below are implemented using pure assembly.
        // Calling them via function pointer can have issues in benches.
        if is_x86_feature_detected!("sse2") {
            // Auto-selected SSE2
            group.bench_with_input(BenchmarkId::new("sse2 (auto)", size), &size, |b, &_size| {
                b.iter(|| unsafe {
                    sse2(
                        black_box(input.as_ptr()),
                        black_box(output.as_mut_ptr()),
                        black_box(input.len()),
                    )
                });
            });

            // Unroll-2 variant
            group.bench_with_input(
                BenchmarkId::new("sse2 unroll-2", size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        punpckhqdq_unroll_2(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );

            // Unroll-4 variant
            group.bench_with_input(
                BenchmarkId::new("sse2 unroll-4", size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        punpckhqdq_unroll_4(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );

            // Unroll-8 variant
            group.bench_with_input(
                BenchmarkId::new("sse2 unroll-8", size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        punpckhqdq_unroll_8(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );
        }

        if is_x86_feature_detected!("avx2") {
            // Auto-selected AVX2
            group.bench_with_input(
                BenchmarkId::new("avx2_permute", size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        avx2_permute(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("avx2_permute_unroll_2", size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        avx2_permute_unroll_2(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("avx2_permute_unroll_4", size),
                &size,
                |b, &_size| {
                    b.iter(|| unsafe {
                        avx2_permute_unroll_4(
                            black_box(input.as_ptr()),
                            black_box(output.as_mut_ptr()),
                            black_box(input.len()),
                        )
                    });
                },
            );
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
