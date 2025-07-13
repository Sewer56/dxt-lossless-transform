use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc3::transform::standard::transform::bench::avx512_vbmi_transform;
use safe_allocator_api::RawAlloc;

fn bench_avx512_vbmi(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_vbmi_transform(
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
    if is_x86_feature_detected!("avx512vbmi") {
        group.bench_with_input(BenchmarkId::new("avx512 vbmi", size), &size, |b, _| {
            bench_avx512_vbmi(b, input, output)
        });
    }

    if !important_benches_only {}
}
