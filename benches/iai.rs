use iai::black_box;
use vmdl::mdl::Mdl;
use vmdl::vtx::Vtx;
use vmdl::vvd::Vvd;

fn parse_mdl() {
    let data = include_bytes!("../data/barrel01.mdl");
    Mdl::read(black_box(data)).unwrap();
}

fn parse_vtx() {
    let data = include_bytes!("../data/barrel01.dx90.vtx");
    Vtx::read(black_box(data)).unwrap();
}

fn parse_vvd() {
    let data = include_bytes!("../data/barrel01.vvd");
    Vvd::read(black_box(data)).unwrap();
}

iai::main!(parse_mdl, parse_vtx, parse_vvd);
