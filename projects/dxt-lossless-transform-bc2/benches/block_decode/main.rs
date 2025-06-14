use core::alloc::Layout;
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_bc2::util::decode_bc2_block;
use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
use safe_allocator_api::RawAlloc;

#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
))]
use pprof::criterion::{Output, PProfProfiler};

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC2 Decode Blocks (BC2 -> RGBA8888)");

    // Set up the test data - 8MB of BC2 blocks
    let bc2_size = 8388608; // 8MB
    let blocks_count = bc2_size / 16; // Each BC2 block is 16 bytes

    // Allocate memory for input BC2 data and output pixels
    let input = allocate_align_64(bc2_size);
    let mut output = allocate_align_64(blocks_count * core::mem::size_of::<Decoded4x4Block>());

    // Initialize input with test data (simple pattern for BC2 blocks)
    unsafe {
        let input_ptr = input.as_ptr() as *mut u8;
        // Fill with valid BC2 blocks
        for block_idx in 0..(bc2_size / 16) {
            let block_ptr = input_ptr.add(block_idx * 16);

            // Write alpha values (4 bits per pixel = 8 bytes total)
            for i in 0..8 {
                *block_ptr.add(i) = ((block_idx * i) % 255) as u8;
            }

            // Write color endpoints (RGB565 format)
            *block_ptr.add(8) = 0x40; // First color (R)
            *block_ptr.add(9) = 0xF8; // First color (G+B)
            *block_ptr.add(10) = 0x00; // Second color (R)
            *block_ptr.add(11) = 0xF8; // Second color (G+B)

            // Write color indices (randomized)
            for i in 12..16 {
                *block_ptr.add(i) = ((block_idx * i) % 255) as u8;
            }
        }
    }

    let input_ptr = input.as_ptr();
    let output_blocks = output.as_mut_ptr() as *mut Decoded4x4Block;
    group.throughput(criterion::Throughput::Bytes(bc2_size as u64));

    // Benchmark the BC2 decoding function with raw pointers
    group.bench_function("decode_bc2_blocks_raw_ptr", |b| {
        b.iter(|| {
            unsafe {
                // Decode blocks one by one.
                for block_idx in 0..blocks_count {
                    let block_ofs = block_idx * 16;

                    // Decode one block at a time using raw pointer-based function
                    *output_blocks.add(block_idx) = decode_bc2_block(input_ptr.add(block_ofs));
                }
            }
        })
    });

    // Benchmark the has_identical_pixels method on decoded blocks
    // Decode all blocks to have data to work with ahead of running bench.
    unsafe {
        for block_idx in 0..blocks_count {
            let block_ofs = block_idx * 16;
            *output_blocks.add(block_idx) = decode_bc2_block(input_ptr.add(block_ofs));
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
