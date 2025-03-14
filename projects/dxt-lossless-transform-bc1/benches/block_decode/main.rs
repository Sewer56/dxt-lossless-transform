use core::alloc::Layout;
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_bc1::util::{decode_bc1_block, DecodedBc1Block};
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

/// Allocates a memory block of the specified size with a 64-byte alignment.
///
/// This function constructs a memory layout with a fixed 64-byte alignment for the given number of bytes
/// and allocates the corresponding memory, returning a `RawAlloc` instance that represents this allocation.
///
/// # Panics
///
/// Panics if creating the memory layout or performing the allocation fails.
///
/// # Examples
///
/// ```
/// let allocation = allocate_align_64(1024);
/// // `allocation` now represents a 1024-byte memory block with 64-byte alignment.
/// ```
pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

/// Benchmarks the decoding of BC1 blocks into RGBA8888 format using raw pointer arithmetic.
///
/// This function creates a benchmark group named "BC1 Decode Blocks (BC1 -> RGBA8888)". It allocates an 8MB input
/// buffer simulating BC1-compressed data and an output buffer for decoded blocks, initialising the input with a
/// repeating byte pattern. The benchmark then iterates over each 8-byte BC1 block, decodes it via a raw pointer–
/// based function, and records the throughput based on the input size.
///
/// # Examples
///
/// ```
/// use criterion::Criterion;
///
/// fn main() {
///     let mut c = Criterion::default();
///     criterion_benchmark(&mut c);
/// }
/// ```
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BC1 Decode Blocks (BC1 -> RGBA8888)");

    // Set up the test data - 8MB of BC1 blocks
    let bc1_size = 8388608; // 8MB
    let blocks_count = bc1_size / 8; // Each BC1 block is 8 bytes

    // Allocate memory for input BC1 data and output pixels
    let input = allocate_align_64(bc1_size);
    let mut output = allocate_align_64(blocks_count * size_of::<DecodedBc1Block>());

    // Initialize input with test data (simple pattern for BC1 blocks)
    unsafe {
        let input_ptr = input.as_ptr() as *mut u8;
        for x in 0..bc1_size {
            // This creates simple BC1 blocks with varying colors
            // Real-world data would have more variety, but this is suitable for benchmarking
            *input_ptr.add(x) = (x % 255) as u8;
        }
    }

    let input_ptr = input.as_ptr();
    let output_blocks = output.as_mut_ptr() as *mut DecodedBc1Block;
    group.throughput(criterion::Throughput::Bytes(bc1_size as u64));

    // Benchmark the BC1 decoding function with raw pointers
    group.bench_function("decode_bc1_blocks_raw_ptr", |b| {
        b.iter(|| {
            unsafe {
                // Decode blocks one by one.
                for block_idx in 0..blocks_count {
                    let block_ofs = block_idx * 8;

                    // Decode one block at a time using raw pointer-based function
                    *output_blocks.add(block_idx) = decode_bc1_block(input_ptr.add(block_ofs));
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
