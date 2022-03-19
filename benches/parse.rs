use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read;
use vmdl::mdl::Mdl;
use vmdl::vtx::Vtx;
use vmdl::vvd::Vvd;

fn parse_mdl(c: &mut Criterion) {
    let data = read("data/barrel01.mdl").unwrap();
    c.bench_function("mdl", |b| b.iter(|| Mdl::read(black_box(&data)).unwrap()));
}

fn parse_vtx(c: &mut Criterion) {
    let data = read("data/barrel01.dx90.vtx").unwrap();
    c.bench_function("vtx", |b| b.iter(|| Vtx::read(black_box(&data)).unwrap()));
}

fn parse_vvd(c: &mut Criterion) {
    let data = read("data/barrel01.vvd").unwrap();
    c.bench_function("vvd", |b| b.iter(|| Vvd::read(black_box(&data)).unwrap()));
}

criterion_group!(benches, parse_mdl, parse_vtx, parse_vvd);
criterion_main!(benches);
