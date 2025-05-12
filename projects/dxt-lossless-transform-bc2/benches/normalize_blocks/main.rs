use core::{alloc::Layout, ptr::copy_nonoverlapping};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_bc2::normalize_blocks::{normalize_blocks, ColorNormalizationMode};
use safe_allocator_api::RawAlloc;
use std::fs;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

// Path to the BC2 file to benchmark with
// You may need to update this path to point to a valid BC2 texture file on your system
const TEST_FILE_PATH: &str =
    "/home/sewer/Temp/texture-stuff/bc2-raw/202x-architecture-10.01/farmhouse/ivy01.dds";

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

#[allow(clippy::needless_range_loop)]
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC2 Normalize Blocks");

    // Try to load the test file
    let file_data = match fs::read(TEST_FILE_PATH) {
        Ok(data) => {
            println!("Successfully loaded test file: {TEST_FILE_PATH}");
            data
        }
        Err(e) => {
            println!("Warning: Could not load test file '{TEST_FILE_PATH}': {e}");
            println!("To run this benchmark, you need a valid BC2 texture file.");
            println!("Please update the TEST_FILE_PATH constant to point to a valid BC2 file or create a sample BC2 file.");
            println!("Alternatively, we'll create a synthetic BC2 file for benchmarking.");

            // Create a synthetic file with 1024 identical BC2 blocks for benchmarking
            // BC2 block is 16 bytes (8 for alpha, 4 for colors, 4 for indices)
            let block_size = 16;
            let num_blocks = 1024;
            let total_size = block_size * num_blocks;

            // Create a sample BC2 block with solid red color
            let mut sample_block = [0u8; 16];

            // Alpha part (first 8 bytes) - all fully opaque (255)
            for x in 0..8 {
                sample_block[x] = 0xFF;
            }

            // Color endpoints (RGB565)
            // Red = 0xF800 in RGB565 (little endian: [0x00, 0xF8])
            sample_block[8] = 0x00;
            sample_block[9] = 0xF8;
            sample_block[10] = 0x00; // Second color not used
            sample_block[11] = 0x00;

            // Indices (all 0)
            sample_block[12] = 0;
            sample_block[13] = 0;
            sample_block[14] = 0;
            sample_block[15] = 0;

            // Create the synthetic file by repeating the sample block
            let mut synthetic_data = Vec::with_capacity(total_size);
            for _ in 0..num_blocks {
                synthetic_data.extend_from_slice(&sample_block);
            }

            println!(
                "Created synthetic BC2 file with {} blocks ({} bytes) for benchmarking.",
                num_blocks,
                synthetic_data.len()
            );
            synthetic_data
        }
    };

    // Ensure we have a file size that's a multiple of 16 (BC2 block size)
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
