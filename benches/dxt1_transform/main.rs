use core::alloc::Layout;
use criterion::{criterion_group, criterion_main, Criterion};
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[cfg(target_feature = "avx2")]
mod avx2;
mod portable32;
mod portable64;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

// Helper to generate test data with predictable patterns
pub(crate) fn generate_test_data(num_blocks: usize) -> RawAlloc {
    let mut data = allocate_align_64(num_blocks * 8);
    let mut data_ptr = data.as_mut_ptr();

    let mut color_byte = 0_u8;
    let mut index_byte = 128_u8;
    unsafe {
        for _ in 0..num_blocks {
            *data_ptr = color_byte.wrapping_add(0);
            *data_ptr.add(1) = color_byte.wrapping_add(1);
            *data_ptr.add(2) = color_byte.wrapping_add(2);
            *data_ptr.add(3) = color_byte.wrapping_add(3);
            color_byte = color_byte.wrapping_add(4);

            *data_ptr.add(4) = index_byte.wrapping_add(0);
            *data_ptr.add(5) = index_byte.wrapping_add(1);
            *data_ptr.add(6) = index_byte.wrapping_add(2);
            *data_ptr.add(7) = index_byte.wrapping_add(3);
            index_byte = index_byte.wrapping_add(4);
            data_ptr = data_ptr.add(8);
        }
    }

    data
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("DXT1 Transform Implementations");
    let size = 16384; // 512x512 px = 16384 blocks
    let input = generate_test_data(size);
    let mut output = allocate_align_64(input.len());
    let important_benches_only = false; // Set to false to enable extra benches, unrolls, etc.

    group.throughput(criterion::Throughput::Bytes(size as u64));

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
            #[cfg(target_feature = "avx2")]
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
