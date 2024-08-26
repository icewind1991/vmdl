use cgmath::Euler;
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

    // dbg!(&mdl.header2);

    // for bone in &mdl.bones {
    //     println!(
    //         "{}: from {} at\n\t{:?}\n\t{:?}\n\t{:?}",
    //         bone.name, bone.parent, bone.rot, bone.q_alignment, bone.pose_to_bone
    //     );
    // }
    dbg!(&mdl.local_animations[0]);
    let transform = mdl
        .local_animations
        .first()
        .map(|a| a.animations[0].rotation(0))
        .unwrap();
    dbg!(transform);
    dbg!(Euler::from(cgmath::Quaternion::from(transform)));

    // dbg!(&mdl.attachments);
    let _model = Model::from_parts(mdl, vtx, vvd);
    // dbg!(Euler::from(Quaternion::from(model.root_transform())));
    // for strip in model.vertex_strips() {
    //     for vertex in strip {
    //         println!("{:?}", vertex);
    //     }
    //     println!("")
    // }

    Ok(())
}
