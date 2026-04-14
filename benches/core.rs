use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mdtools::parser::ParsedDocument;

fn bench_parse_progress(c: &mut Criterion) {
    let source = include_str!("../bench/inputs/t5_progress.md");
    c.bench_function("parse/t5_progress", |b| {
        b.iter(|| ParsedDocument::parse(black_box(source.to_owned())).unwrap())
    });
}

fn bench_parse_scale(c: &mut Criterion) {
    let source = include_str!("../bench/inputs/t19_scale.md");
    c.bench_function("parse/t19_scale", |b| {
        b.iter(|| ParsedDocument::parse(black_box(source.to_owned())).unwrap())
    });
}

fn bench_parse_frontmatter(c: &mut Criterion) {
    let source = include_str!("../bench/inputs/t21_frontmatter.md");
    c.bench_function("parse_for_frontmatter/t21_frontmatter", |b| {
        b.iter(|| ParsedDocument::parse_for_frontmatter(black_box(source.to_owned())).unwrap())
    });
}

criterion_group!(
    benches,
    bench_parse_progress,
    bench_parse_scale,
    bench_parse_frontmatter
);
criterion_main!(benches);
