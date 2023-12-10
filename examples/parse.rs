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

    let models = mdl
        .body_parts
        .iter()
        .flat_map(|part| part.models.iter())
        .flat_map(|model| model.meshes.iter())
        .map(|mesh| mesh.material)
        .collect::<Vec<_>>();
    dbg!(mdl.textures, models, mdl.skin_table);

    // let model = Model::from_parts(mdl, vtx, vvd);
    // for strip in model.vertex_strips() {
    //     for vertex in strip {
    //         println!("{:?}", vertex);
    //     }
    //     println!("")
    // }

    Ok(())
}
