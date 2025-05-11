use core::alloc::Layout;
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_bc3::util::decode_bc3_block;
use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC3 Decode Blocks (BC3 -> RGBA8888)");

    // Set up the test data - 8MB of BC3 blocks
    let bc3_size = 8388608; // 8MB
    let blocks_count = bc3_size / 16; // Each BC3 block is 16 bytes

    // Allocate memory for input BC3 data and output pixels
    let input = allocate_align_64(bc3_size);
    let mut output = allocate_align_64(blocks_count * core::mem::size_of::<Decoded4x4Block>());

    // Initialize input with test data (simple pattern for BC3 blocks)
    unsafe {
        let input_ptr = input.as_ptr() as *mut u8;
        // Fill with valid BC3 blocks
        for block_idx in 0..(bc3_size / 16) {
            let block_ptr = input_ptr.add(block_idx * 16);

            // Write alpha endpoints and indices (BC4 compression for alpha)
            *block_ptr = 255; // Alpha0 (max alpha)
            *block_ptr.add(1) = 0; // Alpha1 (min alpha)

            // Fill alpha indices (3 bits per index)
            for x in 2..8 {
                *block_ptr.add(x) = ((block_idx * x) % 255) as u8;
            }

            // Write color endpoints (RGB565 format - same as BC1/BC2)
            *block_ptr.add(8) = 0x40; // First color (R)
            *block_ptr.add(9) = 0xF8; // First color (G+B)
            *block_ptr.add(10) = 0x00; // Second color (R)
            *block_ptr.add(11) = 0xF8; // Second color (G+B)

            // Write color indices (randomized)
            for x in 12..16 {
                *block_ptr.add(x) = ((block_idx * x) % 255) as u8;
            }
        }
    }

    let input_ptr = input.as_ptr();
    let output_blocks = output.as_mut_ptr() as *mut Decoded4x4Block;
    group.throughput(criterion::Throughput::Bytes(bc3_size as u64));

    // Benchmark the BC3 decoding function with raw pointers
    group.bench_function("decode_bc3_blocks_raw_ptr", |b| {
        b.iter(|| {
            unsafe {
                // Decode blocks one by one.
                for block_idx in 0..blocks_count {
                    let block_ofs = block_idx * 16;

                    // Decode one block at a time using raw pointer-based function
                    *output_blocks.add(block_idx) = decode_bc3_block(input_ptr.add(block_ofs));
                }
            }
        })
    });

    // Benchmark the has_identical_pixels method on decoded blocks
    // Decode all blocks to have data to work with ahead of running bench.
    unsafe {
        for block_idx in 0..blocks_count {
            let block_ofs = block_idx * 16;
            *output_blocks.add(block_idx) = decode_bc3_block(input_ptr.add(block_ofs));
        }
    }

    group.bench_function("has_identical_pixels", |b| {
        unsafe {
            // Then benchmark the has_identical_pixels method
            b.iter(|| {
                let mut identical_count = 0;
                for block_idx in 0..blocks_count {
                    if (*output_blocks.add(block_idx)).has_identical_pixels() {
                        identical_count += 1;
                    }
                }
                identical_count
            })
        }
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
