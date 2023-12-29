use std::env::args;
use std::fs;
use std::path::PathBuf;
use vmdl::mdl::Mdl;
use vmdl::vtx::Vtx;
use vmdl::vvd::Vvd;

fn main() -> Result<(), vmdl::ModelError> {
    let mut args = args();
    let _ = args.next();
    let path = PathBuf::from(args.next().expect("No demo file provided"));

    let data = fs::read(&path)?;
    let mdl = Mdl::read(&data)?;
    let data = fs::read(path.with_extension("dx90.vtx"))?;
    let _vtx = Vtx::read(&data)?;
    let data = fs::read(path.with_extension("vvd"))?;
    let _vvd = Vvd::read(&data)?;

    for bone in mdl.bones {
        println!("{}: from {} at {:?}", bone.name, bone.parent, bone.rot);
    }

    // let model = Model::from_parts(mdl, vtx, vvd);
    // for strip in model.vertex_strips() {
    //     for vertex in strip {
    //         println!("{:?}", vertex);
    //     }
    //     println!("")
    // }

    Ok(())
}
