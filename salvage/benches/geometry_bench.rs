use criterion::{criterion_group, criterion_main, Criterion};
use salvage::{
    callback::{InitCallback, Initialization},
    geometry::SvgSet,
};
use usvg::Node;

fn criterion_benchmark(c: &mut Criterion) {
    let callback_fn = |node: &Node| -> Initialization { Initialization::default() };
    let callback = InitCallback::new(callback_fn);
    let mut svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);
    c.bench_function("update text", |b| {
        b.iter(|| svg_set.update_text(&"instruction #dynamicText".to_string(), &"hi".to_string()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
