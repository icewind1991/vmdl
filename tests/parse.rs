use std::fs::read;
use vmdl::mdl::Mdl;
use vmdl::vtx::Vtx;
use vmdl::vvd::Vvd;
use vmdl::Model;

#[test]
fn parse_mdl() {
    let data = read("data/barrel01.mdl").unwrap();
    Mdl::read(&data).unwrap();
}

#[test]
fn parse_vtx() {
    let data = read("data/barrel01.dx90.vtx").unwrap();
    Vtx::read(&data).unwrap();
}

#[test]
fn parse_vvd() {
    let data = read("data/barrel01.vvd").unwrap();
    Vvd::read(&data).unwrap();
}
