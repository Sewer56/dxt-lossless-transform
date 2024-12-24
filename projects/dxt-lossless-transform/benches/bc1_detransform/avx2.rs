use criterion::*;
use dxt_lossless_transform::raw::detransform::avx2::*;
use safe_allocator_api::RawAlloc;

fn bench_unpck(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        unpck_detransform(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_unpck_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        unpck_detransform_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

#[cfg(target_arch = "x86_64")]
fn bench_unpck_unroll_4(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        unpck_detransform_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

pub(crate) fn run_benchmarks(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    input: &RawAlloc,
    output: &mut RawAlloc,
    size: usize,
    important_benches_only: bool,
) {
    group.bench_with_input(
        BenchmarkId::new("avx2 unpck unroll 2", size),
        &size,
        |b, _| bench_unpck_unroll_2(b, input, output),
    );

    if !important_benches_only {
        group.bench_with_input(BenchmarkId::new("avx2 unpck", size), &size, |b, _| {
            bench_unpck(b, input, output)
        });

        #[cfg(target_arch = "x86_64")]
        group.bench_with_input(
            BenchmarkId::new("avx2 unpck unroll 4", size),
            &size,
            |b, _| bench_unpck_unroll_4(b, input, output),
        );
    }
}
