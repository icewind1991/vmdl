use cgmath::Matrix4;
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

    dbg!(Matrix4::from(mdl.bones[1].pose_to_bone.clone()));

    // for animation_desc in &mdl.local_animations {
    //     println!(
    //         "{}: {} frames at {}fps",
    //         animation_desc.name, animation_desc.frame_count, animation_desc.fps,
    //     );
    //     for animation in &animation_desc.animations {
    //         println!(
    //             "\tbone {:.2} frame 0:\n\t\trot: {:?}\n\t\tpos: {:?}",
    //             animation.bone,
    //             animation.rotation(0),
    //             animation.position(0),
    //         );
    //         println!(
    //             "\tbone {:.2} frame 1:\n\t\trot: {:?}\n\t\tpos: {:?}",
    //             animation.bone,
    //             animation.rotation(10),
    //             animation.position(10),
    //         );
    //     }
    // }

    let model = Model::from_parts(mdl, vtx, vvd);

    let _ = model;

    Ok(())
}
