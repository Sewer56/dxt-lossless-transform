use core::{alloc::Layout, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion};
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;
mod portable32;
mod portable64;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC2 Transform Implementations");
    let size = 8388608; // bc2 = 1byte/pixel, so this is 4096x2048px. Keep this size for comparison with BC1.
    let input = allocate_align_64(size);
    let mut output = allocate_align_64(input.len());
    let important_benches_only = true; // Set to false to enable extra benches, unrolls, etc.

    group.throughput(criterion::Throughput::Bytes(size as u64));
    group.warm_up_time(Duration::from_secs(60));
    group.measurement_time(Duration::from_secs(60));

    // Run architecture-specific benchmarks
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        if is_x86_feature_detected!("sse2") {
            sse2::run_benchmarks(
                &mut group,
                &input,
                &mut output,
                size,
                important_benches_only,
            );
        }

        if is_x86_feature_detected!("avx2") {
            avx2::run_benchmarks(
                &mut group,
                &input,
                &mut output,
                size,
                important_benches_only,
            );
        }
    }

    // Run all portable benchmarks
    portable32::run_benchmarks(
        &mut group,
        &input,
        &mut output,
        size,
        important_benches_only,
    );
    portable64::run_benchmarks(
        &mut group,
        &input,
        &mut output,
        size,
        important_benches_only,
    );

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
