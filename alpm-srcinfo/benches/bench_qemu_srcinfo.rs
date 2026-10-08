//! QEMU .SRCINFO, since it is huge!

use std::hint::black_box;

use alpm_srcinfo::SourceInfoV1;
use criterion::{Criterion, criterion_group, criterion_main};
use testresult::TestResult;

/// Benchmarks the parsing of a large SRCINFO file.
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse QEMU .SRCINFO");

    group.bench_function("from embedded &'static str", |b| {
        b.iter(|| {
            let source_info = SourceInfoV1::from_string(black_box(include_str!("qemu.SRCINFO")))?;
            black_box(source_info);

            TestResult::Ok(())
        })
    });

    group.bench_function("from file", |b| {
        b.iter(|| {
            const SRCINFO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/qemu.SRCINFO");
            let source_info = SourceInfoV1::from_file(SRCINFO_PATH.as_ref())?;
            black_box(source_info);

            TestResult::Ok(())
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
