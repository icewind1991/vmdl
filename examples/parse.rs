use std::env::args;
use std::fs;
use std::path::PathBuf;
use vmdl::mdl::Mdl;
use vmdl::vtx::Vtx;
use vmdl::vvd::Vvd;
use vmdl::Model;

fn main() -> Result<(), vmdl::ModelError> {
    let mut args = args();
    let _ = args.next();
    let path = PathBuf::from(args.next().expect("No demo file provided"));

    let data = fs::read(&path)?;
    let mdl = Mdl::read(&data)?;
    let data = fs::read(path.with_extension("dx90.vtx"))?;
    let vtx = Vtx::read(&data)?;
    let data = fs::read(path.with_extension("vvd"))?;
    let vvd = Vvd::read(&data)?;

    for bone in &mdl.bones {
        println!(
            "{}: from {} at\n\t{:?}\n\t{:?}\n\t{:?}\n\t{:?}",
            bone.name, bone.parent, bone.rot, bone.rot_scale, bone.quaternion, bone.pose_to_bone
        );
    }

    let model = Model::from_parts(mdl, vtx, vvd);
    dbg!(model.root_transform());
    dbg!(model.idle_transform());

    Ok(())
}
