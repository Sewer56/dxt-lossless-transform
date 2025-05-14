use core::{alloc::Layout, ptr::copy_nonoverlapping};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_bc1::normalize_blocks::{
    normalize_blocks, normalize_blocks_all_modes, ColorNormalizationMode,
};
use safe_allocator_api::RawAlloc;
use std::fs;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

// Path to the BC1 file to benchmark with
const TEST_FILE_PATH: &str = "/home/sewer/Downloads/texture-stuff/bc1-raw/202x-architecture-10.01/whiterun/wrwoodlattice01.dds";

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC1 Normalize Blocks");

    // Try to load the test file
    let file_data = match fs::read(TEST_FILE_PATH) {
        Ok(data) => {
            println!("Successfully loaded test file: {TEST_FILE_PATH}");
            data
        }
        Err(_) => {
            println!("Warning: Could not load test file '{TEST_FILE_PATH}'. Exiting.");
            return;
        }
    };

    // Ensure we have a file size that's a multiple of 8 (BC1 block size)
    let file_size = file_data.len();

    // Allocate memory for input and output
    let mut input = allocate_align_64(file_size);
    let mut output = allocate_align_64(file_size);

    // Copy file data to input buffer
    unsafe {
        copy_nonoverlapping(file_data.as_ptr(), input.as_mut_ptr(), file_size);
    }

    let input_ptr = input.as_ptr();
    let output_ptr = output.as_mut_ptr();

    group.throughput(criterion::Throughput::Bytes(file_size as u64));

    // Benchmark the normalize_blocks function
    group.bench_function("normalize_blocks", |b| {
        b.iter(|| unsafe {
            normalize_blocks(
                input_ptr,
                output_ptr,
                file_size,
                ColorNormalizationMode::Color0Only,
            );
        })
    });

    // Benchmark normalize_blocks_all_modes
    // Create buffers for each normalization mode outside the benchmark
    let mode_count = ColorNormalizationMode::all_values().len();
    let mut output_buffers = Vec::with_capacity(mode_count);

    for _ in 0..mode_count {
        output_buffers.push(allocate_align_64(file_size));
    }

    group.bench_function("normalize_blocks_all_modes", |b| {
        b.iter(|| unsafe {
            // Create a fresh stack array of pointers for each iteration (else it segfaults)
            // The ColorNormalizationMode enum has three variants: None, Color0Only, ReplicateColor
            let mut output_ptrs_array = [
                output_buffers[0].as_mut_ptr(), // None
                output_buffers[1].as_mut_ptr(), // Color0Only
                output_buffers[2].as_mut_ptr(), // ReplicateColor
            ];

            // Run the function
            normalize_blocks_all_modes(input_ptr, &mut output_ptrs_array, file_size);
        })
    });

    group.finish();
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
