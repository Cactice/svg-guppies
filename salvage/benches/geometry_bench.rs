use criterion::{criterion_group, criterion_main, Criterion};
use salvage::{
    callback::{InitCallback, PassDown},
    svg_set::SvgSet,
};

fn criterion_benchmark(c: &mut Criterion) {
    let callback = InitCallback::new(|_| (None, PassDown::default()));
    let mut svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);
    c.bench_function("update text", |b| {
        b.iter(|| svg_set.update_text(&"instruction #dynamicText".to_string(), &"hi".to_string()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
