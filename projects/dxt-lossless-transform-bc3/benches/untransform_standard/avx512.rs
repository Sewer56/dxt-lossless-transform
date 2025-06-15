use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc3::transforms::standard::untransform::bench_exports::{
    avx512_detransform, avx512_detransform_32_vbmi, avx512_detransform_32_vl,
};
use safe_allocator_api::RawAlloc;

fn bench_avx512_32_vbmi(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_detransform_32_vbmi(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx512_32_vl(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_detransform_32_vl(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx512_64(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_detransform(
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
        BenchmarkId::new("avx512 32bit vbmi", size),
        &size,
        |b, _| bench_avx512_32_vbmi(b, input, output),
    );

    if is_x86_feature_detected!("avx512vl") {
        group.bench_with_input(BenchmarkId::new("avx512 32bit vl", size), &size, |b, _| {
            bench_avx512_32_vl(b, input, output)
        });
    }

    group.bench_with_input(BenchmarkId::new("avx512 64bit", size), &size, |b, _| {
        bench_avx512_64(b, input, output)
    });

    if !important_benches_only {}
}
