use core::{alloc::Layout, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_common::cpu_detect::*;
use safe_allocator_api::RawAlloc;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;
mod generic;

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

/// Output buffers for with_split_alphas transform benchmarks
pub(crate) struct SplitAlphasOutputBuffers {
    pub alpha0: RawAlloc,
    pub alpha1: RawAlloc,
    pub alpha_indices: RawAlloc,
    pub colors: RawAlloc,
    pub color_indices: RawAlloc,
}

impl SplitAlphasOutputBuffers {
    pub fn new(block_count: usize) -> Self {
        Self {
            alpha0: allocate_align_64(block_count), // 1 byte per block
            alpha1: allocate_align_64(block_count), // 1 byte per block
            alpha_indices: allocate_align_64(block_count * 6), // 6 bytes per block
            colors: allocate_align_64(block_count * 4), // 4 bytes per block
            color_indices: allocate_align_64(block_count * 4), // 4 bytes per block
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC3 Split Blocks (Split Alphas)");
    let size = 8388608; // bc3 = 16 bytes per block, so this is 524288 blocks
    let block_count = size / 16;
    let input = allocate_align_64(size);
    let mut output = SplitAlphasOutputBuffers::new(block_count);
    let important_benches_only = false; // Set to false to enable extra benches, unrolls, etc.

    group.throughput(criterion::Throughput::Bytes(size as u64));
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));

    // Run architecture-specific benchmarks
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        if has_avx2() {
            avx2::run_benchmarks(
                &mut group,
                &input,
                &mut output,
                size,
                block_count,
                important_benches_only,
            );
        }
    }

    // Run all portable benchmarks
    generic::run_benchmarks(
        &mut group,
        &input,
        &mut output,
        size,
        block_count,
        important_benches_only,
    );

    group.finish();

    #[cfg(not(feature = "pgo"))]
    {
        // Benchmarks excluded from PGO run.
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(benches);
