use core::mem::size_of;
use core::{alloc::Layout, slice, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion};
use dxt_lossless_transform_common::color_565::Color565;
use safe_allocator_api::RawAlloc;

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
    fn from_slice_with_transform(
        source: &[Color565],
        transform_fn: fn(&[Color565], &mut [Color565]),
    ) -> Self {
        let mut buffer = Self::from_slice(source);
        transform_fn(source, buffer.as_mut_slice());
        buffer
    }

    /// Returns a raw pointer to the underlying slice
    fn as_mut_ptr(&mut self) -> *mut Color565 {
        unsafe { (*self.colors).as_mut_ptr() }
    }

    /// Returns a raw pointer to the underlying slice
    fn as_ptr(&self) -> *const Color565 {
        unsafe { (*self.colors).as_ptr() }
    }

    /// Get a mutable reference to the underlying slice
    fn as_mut_slice(&mut self) -> &mut [Color565] {
        unsafe { &mut *self.colors }
    }

    /// Number of items (colors) in the buffer.
    /// (Not number of bytes.)
    fn len(&self) -> usize {
        self.colors.len()
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
    let mut output_buffer = Color565Buffer::from_slice(input_colors);

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
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("decorrelate_ycocg_r_var1_ptr", |b| {
        b.iter(|| unsafe {
            Color565::decorrelate_ycocg_r_var1_ptr(
                input_colors.as_ptr(),
                output_buffer.as_mut_ptr(),
                input_colors.len(),
            );
        })
    });

    group.bench_function("decorrelate_ycocg_r_var2_ptr", |b| {
        b.iter(|| unsafe {
            Color565::decorrelate_ycocg_r_var2_ptr(
                input_colors.as_ptr(),
                output_buffer.as_mut_ptr(),
                input_colors.len(),
            );
        })
    });

    group.bench_function("decorrelate_ycocg_r_var3_ptr", |b| {
        b.iter(|| unsafe {
            Color565::decorrelate_ycocg_r_var3_ptr(
                input_colors.as_ptr(),
                output_buffer.as_mut_ptr(),
                input_colors.len(),
            );
        })
    });

    // Create decorrelated arrays for recorrelation benchmarks
    let mut decorrelated_var1 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var1_slice,
    );

    group.bench_function("recorrelate_ycocg_r_var1_ptr", |b| {
        b.iter(|| unsafe {
            Color565::recorrelate_ycocg_r_var1_ptr(
                decorrelated_var1.as_ptr(),
                decorrelated_var1.as_mut_ptr(),
                decorrelated_var1.len(),
            );
        })
    });

    drop(decorrelated_var1);
    let mut decorrelated_var2 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var2_slice,
    );

    group.bench_function("recorrelate_ycocg_r_var2_ptr", |b| {
        b.iter(|| unsafe {
            Color565::recorrelate_ycocg_r_var2_ptr(
                decorrelated_var2.as_ptr(),
                decorrelated_var2.as_mut_ptr(),
                decorrelated_var2.len(),
            );
        })
    });

    drop(decorrelated_var2);
    let mut decorrelated_var3 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var3_slice,
    );
    group.bench_function("recorrelate_ycocg_r_var3_ptr", |b| {
        b.iter(|| unsafe {
            Color565::recorrelate_ycocg_r_var3_ptr(
                decorrelated_var3.as_ptr(),
                decorrelated_var3.as_mut_ptr(),
                decorrelated_var3.len(),
            );
        })
    });

    // Test split and recorrelate at the same time.
    let decorrelated_var1 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var1_slice,
    );
    let mut decorrelated_dest = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var1_slice,
    );

    group.bench_function("recorrelate_ycocg_r_var1_ptr_split", |b| {
        b.iter(|| unsafe {
            Color565::recorrelate_ycocg_r_var1_ptr_split(
                decorrelated_var1.as_ptr(),
                decorrelated_var1.as_ptr().add(decorrelated_var1.len() / 2),
                decorrelated_dest.as_mut_ptr(), // use different destination pointer and non-overlapping buffers.
                decorrelated_var1.len(),
            );
        })
    });

    drop(decorrelated_var1);
    let decorrelated_var2 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var2_slice,
    );
    group.bench_function("recorrelate_ycocg_r_var2_ptr_split", |b| {
        b.iter(|| unsafe {
            Color565::recorrelate_ycocg_r_var2_ptr_split(
                decorrelated_var2.as_ptr(),
                decorrelated_var2.as_ptr().add(decorrelated_var2.len() / 2),
                decorrelated_dest.as_mut_ptr(), // use different destination pointer and non-overlapping buffers.
                decorrelated_var2.len(),
            );
        })
    });

    drop(decorrelated_var2);
    let decorrelated_var3 = Color565Buffer::from_slice_with_transform(
        input_colors,
        Color565::decorrelate_ycocg_r_var3_slice,
    );

    group.bench_function("recorrelate_ycocg_r_var3_ptr_split", |b| {
        b.iter(|| unsafe {
            Color565::recorrelate_ycocg_r_var3_ptr_split(
                decorrelated_var3.as_ptr(),
                decorrelated_var3.as_ptr().add(decorrelated_var3.len() / 2),
                decorrelated_dest.as_mut_ptr(), // use different destination pointer and non-overlapping buffers.
                decorrelated_var3.len(),
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
