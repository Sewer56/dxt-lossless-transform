use core::{alloc::Layout, slice};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_common::color_565::Color565;
use safe_allocator_api::RawAlloc;
use std::mem::size_of;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

// A wrapper struct that owns both the allocation and provides access to the slice
struct Color565Buffer {
    _alloc: RawAlloc, // Ownership kept in struct, not meant to be accessed directly
    colors: *mut [Color565],
}

impl Color565Buffer {
    // Create a new buffer with the specified number of items
    fn new(num_items: usize) -> Self {
        let mut alloc = allocate_align_64(num_items * size_of::<Color565>());
        let colors =
            unsafe { slice::from_raw_parts_mut(alloc.as_mut_ptr() as *mut Color565, num_items) };

        // Transfer the slice to a raw pointer
        let colors_ptr = colors as *mut [Color565];

        Self {
            _alloc: alloc,
            colors: colors_ptr,
        }
    }

    // Create a buffer and copy data from another slice
    fn from_slice(source: &[Color565]) -> Self {
        let buffer = Self::new(source.len());
        unsafe {
            (*buffer.colors).copy_from_slice(source);
        }
        buffer
    }

    // Create a buffer, copy data, and apply a transformation function
    fn from_slice_with_transform(source: &[Color565], transform_fn: fn(&mut [Color565])) -> Self {
        let mut buffer = Self::from_slice(source);
        transform_fn(buffer.as_mut_slice());
        buffer
    }

    // Get a mutable reference to the underlying slice
    fn as_mut_slice(&mut self) -> &mut [Color565] {
        unsafe { &mut *self.colors }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Color565 YCoCg-R Conversion");

    // Set up test data - array of Color565 values to convert
    const BYTES_PER_ITEM: usize = size_of::<Color565>();
    const NUM_ITEMS: usize = 2_000_000; // 2 million colors, equivalent to 8MiB DXT1 texture
                                        // 2M * 2 bytes = 4MiB == 50% of a 8MiB DXT1 texture

    // Allocate memory for input data
    let mut input_buffer = Color565Buffer::new(NUM_ITEMS);
    let input_colors = input_buffer.as_mut_slice();

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

    // Benchmark for decorrelate_ycocg_r_var1_slice
    group.bench_function("decorrelate_ycocg_r_var1_slice", |b| {
        b.iter_batched(
            || Color565Buffer::from_slice(input_colors),
            |mut buffer| {
                Color565::decorrelate_ycocg_r_var1_slice(buffer.as_mut_slice());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark for decorrelate_ycocg_r_var2_slice
    group.bench_function("decorrelate_ycocg_r_var2_slice", |b| {
        b.iter_batched(
            || Color565Buffer::from_slice(input_colors),
            |mut buffer| {
                Color565::decorrelate_ycocg_r_var2_slice(buffer.as_mut_slice());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark for decorrelate_ycocg_r_var3_slice
    group.bench_function("decorrelate_ycocg_r_var3_slice", |b| {
        b.iter_batched(
            || Color565Buffer::from_slice(input_colors),
            |mut buffer| {
                Color565::decorrelate_ycocg_r_var3_slice(buffer.as_mut_slice());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Create decorrelated arrays for recorrelation benchmarks
    let mut decorrelated_var1 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var1_slice,
    );
    let mut decorrelated_var2 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var2_slice,
    );
    let mut decorrelated_var3 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var3_slice,
    );

    // Benchmark for recorrelate_ycocg_r_var1_slice
    group.bench_function("recorrelate_ycocg_r_var1_slice", |b| {
        b.iter_batched(
            || Color565Buffer::from_slice(decorrelated_var1.as_mut_slice()),
            |mut buffer| {
                Color565::recorrelate_ycocg_r_var1_slice(buffer.as_mut_slice());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark for recorrelate_ycocg_r_var2_slice
    group.bench_function("recorrelate_ycocg_r_var2_slice", |b| {
        b.iter_batched(
            || Color565Buffer::from_slice(decorrelated_var2.as_mut_slice()),
            |mut buffer| {
                Color565::recorrelate_ycocg_r_var2_slice(buffer.as_mut_slice());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark for recorrelate_ycocg_r_var3_slice
    group.bench_function("recorrelate_ycocg_r_var3_slice", |b| {
        b.iter_batched(
            || Color565Buffer::from_slice(decorrelated_var3.as_mut_slice()),
            |mut buffer| {
                Color565::recorrelate_ycocg_r_var3_slice(buffer.as_mut_slice());
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
