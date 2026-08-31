use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mdtools::document::Document;
use mdtools::patch::{Patch, PatchOp, TaskPatchTarget};
use mdtools::target::{TargetKind, TargetQuery};
use mdtools::TaskStatus;

fn bench_parse_progress(c: &mut Criterion) {
    let source = include_str!("../bench/inputs/t5_progress.md");
    c.bench_function("parse/t5_progress", |b| {
        b.iter(|| Document::parse(black_box(source.to_owned())).unwrap())
    });
}

fn bench_parse_scale(c: &mut Criterion) {
    let source = include_str!("../bench/inputs/t19_scale.md");
    c.bench_function("parse/t19_scale", |b| {
        b.iter(|| Document::parse(black_box(source.to_owned())).unwrap())
    });
}

fn bench_parse_frontmatter(c: &mut Criterion) {
    let source = include_str!("../bench/inputs/t21_frontmatter.md");
    c.bench_function("parse_for_frontmatter/t21_frontmatter", |b| {
        b.iter(|| Document::parse_for_frontmatter(black_box(source.to_owned())).unwrap())
    });
}

fn gap_heavy_fixture(count: usize) -> String {
    (0..count)
        .map(|index| format!("paragraph {index}\n\n[^unused-{index}]: omitted {index}\n\n"))
        .collect()
}

fn bench_parse_gap_heavy(c: &mut Criterion) {
    let source = gap_heavy_fixture(256);
    c.bench_function("parse/gap_heavy_256", |b| {
        b.iter(|| Document::parse(black_box(source.clone())).unwrap())
    });
}

fn bench_target_map_scale(c: &mut Criterion) {
    let document = Document::parse(include_str!("../bench/inputs/t19_scale.md")).unwrap();
    c.bench_function("target_map/t19_scale", |b| {
        b.iter(|| black_box(&document).map().unwrap())
    });
}

fn bench_target_query_progress(c: &mut Criterion) {
    let document = Document::parse(include_str!("../bench/inputs/t5_progress.md")).unwrap();
    let query = TargetQuery::Kind {
        kind: TargetKind::Task,
    };
    c.bench_function("target_query_tasks/t5_progress", |b| {
        b.iter(|| black_box(&document).query(black_box(&query)).unwrap())
    });
}

fn bench_target_resolve_read_scale(c: &mut Criterion) {
    let document = Document::parse(include_str!("../bench/inputs/t19_scale.md")).unwrap();
    let address = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Section)
        .expect("scale fixture has a section")
        .address;
    c.bench_function("target_resolve_read/t19_scale", |b| {
        b.iter(|| {
            black_box(&document)
                .resolve(black_box(&address))
                .unwrap()
                .read(black_box(&document))
                .unwrap()
        })
    });
}

fn flat_headings(count: usize) -> String {
    (0..count)
        .map(|index| format!("# Heading {index}\n\nbody {index}\n\n"))
        .collect()
}

fn bench_target_map_flat_headings(c: &mut Criterion) {
    let mut group = c.benchmark_group("target_map_flat_headings");
    for count in [1_000, 2_000, 4_000] {
        let document = Document::parse(flat_headings(count)).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &document,
            |b, document| b.iter(|| black_box(document).map().unwrap()),
        );
    }
    group.finish();
}

fn bench_target_resolve_flat_headings(c: &mut Criterion) {
    let document = Document::parse(flat_headings(4_000)).unwrap();
    let address = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Section)
        .expect("flat fixture has a section")
        .address;
    c.bench_function("target_resolve/flat_4000", |b| {
        b.iter(|| black_box(&document).resolve(black_box(&address)).unwrap())
    });
}

fn task_fixture(count: usize) -> String {
    (0..count)
        .map(|index| format!("- [ ] task {index}\n"))
        .collect()
}

fn bench_patch_task_batches(c: &mut Criterion) {
    let mut group = c.benchmark_group("patch_task_batch");
    for count in [10, 100, 500] {
        let document = Document::parse(task_fixture(count)).unwrap();
        let operations = document
            .map()
            .unwrap()
            .iter()
            .filter(|snapshot| snapshot.kind == TargetKind::Task)
            .map(|snapshot| PatchOp::SetTaskStatus {
                target: TaskPatchTarget::try_from(snapshot).unwrap(),
                status: TaskStatus::Done,
            })
            .collect();
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations,
        };
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            b.iter(|| black_box(&patch).apply(black_box(&document)).unwrap())
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_progress,
    bench_parse_scale,
    bench_parse_frontmatter,
    bench_parse_gap_heavy,
    bench_target_map_scale,
    bench_target_query_progress,
    bench_target_resolve_read_scale,
    bench_target_map_flat_headings,
    bench_target_resolve_flat_headings,
    bench_patch_task_batches
);
criterion_main!(benches);
