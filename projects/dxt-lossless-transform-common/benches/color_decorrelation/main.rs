use core::{alloc::Layout, slice};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_common::color_565::Color565;
use safe_allocator_api::RawAlloc;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Color565 YCoCg-R Conversion");

    // Set up test data - array of Color565 values to convert
    const BYTES_PER_ITEM: usize = size_of::<Color565>();
    const NUM_ITEMS: usize = 2_000_000; // 2 million colors, equivalent to 8MiB DXT1 texture
                                        // 2M * 2 bytes = 4MiB == 50% of a 8MiB DXT1 texture

    // Allocate memory for input data
    let mut input = allocate_align_64(NUM_ITEMS * BYTES_PER_ITEM);
    let input_colors =
        unsafe { slice::from_raw_parts_mut(input.as_mut_ptr() as *mut Color565, NUM_ITEMS) };

    // Fill with test data - sequential RGB565 values
    for (x, color) in input_colors.iter_mut().enumerate() {
        let r = ((x) % 32) as u8; // 5 bits (0-31)
        let g = ((x >> 8) % 32) as u8; // 5 bits (0-31)
        let b = ((x >> 16) % 32) as u8; // 5 bits (0-31)
        *color = Color565::from_rgb(r, g, b);
    }

    group.throughput(criterion::Throughput::Bytes(
        (NUM_ITEMS * BYTES_PER_ITEM) as u64,
    ));
    group.bench_function("decorrelate_ycocg_r", |b| {
        b.iter_batched(
            // Setup: create a fresh copy for each iteration
            || {
                let mut clone = allocate_align_64(NUM_ITEMS * BYTES_PER_ITEM);
                let clone_colors = unsafe {
                    slice::from_raw_parts_mut(clone.as_mut_ptr() as *mut Color565, NUM_ITEMS)
                };
                clone_colors.copy_from_slice(input_colors);
                (clone, clone_colors)
            },
            // Benchmark: run on the fresh copy
            |(_, clone_colors)| {
                for color in clone_colors.iter_mut() {
                    color.decorrelate_ycocg_r_var2();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Create a second test array for the recorrelation benchmark
    let mut decorrelated = allocate_align_64(NUM_ITEMS * BYTES_PER_ITEM);
    let decorrelated_colors =
        unsafe { slice::from_raw_parts_mut(decorrelated.as_mut_ptr() as *mut Color565, NUM_ITEMS) };

    for (x, color) in decorrelated_colors.iter_mut().enumerate() {
        *color = input_colors[x];
        color.decorrelate_ycocg_r_var2();
    }

    group.bench_function("recorrelate_ycocg_r", |b| {
        b.iter_batched(
            // Setup: create a fresh copy for each iteration
            || {
                let mut clone = allocate_align_64(NUM_ITEMS * BYTES_PER_ITEM);
                let clone_colors = unsafe {
                    slice::from_raw_parts_mut(clone.as_mut_ptr() as *mut Color565, NUM_ITEMS)
                };
                clone_colors.copy_from_slice(decorrelated_colors);
                (clone, clone_colors)
            },
            // Benchmark: run on the fresh copy
            |(_, clone_colors)| {
                for color in clone_colors.iter_mut() {
                    color.recorrelate_ycocg_r_var2();
                }
            },
            criterion::BatchSize::SmallInput,
        )
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
