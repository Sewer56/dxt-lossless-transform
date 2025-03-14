use core::{alloc::Layout, slice};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_bc1::util::decode_bc1_block;
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC1 Decode Blocks");

    // Set up the test data - 8MB of BC1 blocks
    let bc1_size = 8388608; // 8MB
    let blocks_count = bc1_size / 8; // Each BC1 block is 8 bytes
    let pixels_count = blocks_count * 16; // Each block decodes to 16 RGBA pixels

    // Allocate memory for input BC1 data and output pixels
    let input = allocate_align_64(bc1_size);
    let mut output = allocate_align_64(pixels_count * 4); // 4 bytes per pixel (RGBA)

    // Initialize input with test data (simple pattern for BC1 blocks)
    unsafe {
        let input_ptr = input.as_ptr() as *mut u8;
        for i in 0..bc1_size {
            // This creates simple BC1 blocks with varying colors
            // Real-world data would have more variety, but this is suitable for benchmarking
            *input_ptr.add(i) = (i % 255) as u8;
        }
    }

    group.throughput(criterion::Throughput::Bytes(bc1_size as u64));

    // Benchmark the BC1 decoding function
    group.bench_function("decode_bc1_blocks", |b| {
        b.iter(|| {
            unsafe {
                let input_ptr = input.as_ptr();
                let output_ptr = output.as_mut_ptr() as *mut u32;

                // Process all blocks
                for i in 0..blocks_count {
                    let block_offset = i * 8;
                    let pixel_offset = i * 16;

                    // Decode one block at a time (4x4 pixels)
                    decode_bc1_block(
                        slice::from_raw_parts(input_ptr.add(block_offset), 8),
                        slice::from_raw_parts_mut(output_ptr.add(pixel_offset), 16),
                        4, // Stride of 4 pixels for a 4x4 block
                    );
                }
            }
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
