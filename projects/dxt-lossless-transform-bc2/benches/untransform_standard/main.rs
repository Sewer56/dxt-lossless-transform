use core::{alloc::Layout, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_common::cpu_detect::*;
use safe_allocator_api::RawAlloc;

#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
))]
use pprof::criterion::{Output, PProfProfiler};

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[cfg(feature = "nightly")]
mod avx512;
mod portable32;
mod portable64;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC2 Unsplit Blocks");
    let size = 8388608; // 4096x4096px
    let input = allocate_align_64(size);
    let mut output = allocate_align_64(input.len());
    let important_benches_only = true; // Set to false to enable extra benches, unrolls, etc.

    group.throughput(criterion::Throughput::Bytes(size as u64));
    group.warm_up_time(Duration::from_secs(60));
    group.measurement_time(Duration::from_secs(60));

    // Run architecture-specific benchmarks
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        if has_sse2() {
            sse2::run_benchmarks(
                &mut group,
                &input,
                &mut output,
                size,
                important_benches_only,
            );
        }

        if has_avx2() {
            avx2::run_benchmarks(
                &mut group,
                &input,
                &mut output,
                size,
                important_benches_only,
            );
        }

        #[cfg(feature = "nightly")]
        if has_avx512f() {
            avx512::run_benchmarks(
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

#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(not(all(
    any(target_os = "linux", target_os = "macos"),
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
)))]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(benches);
