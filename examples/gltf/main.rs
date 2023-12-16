mod convert;
mod error;
mod loader;
mod material;

use gltf_json as json;

use std::fs;

use crate::convert::{push_material, push_model};
use crate::loader::Loader;
use crate::material::load_material_fallback;
pub use error::Error;
use gltf_json::Index;
use std::borrow::Cow;
use std::env::args_os;
use std::io::Write;
use std::path::{Path, PathBuf};
use vmdl::{Mdl, Model, Vtx, Vvd};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Output {
    /// Output standard glTF.
    Standard,

    /// Output binary glTF.
    Binary,
}

fn align_to_multiple_of_four(n: &mut u32) {
    *n = (*n + 3) & !3;
}

fn pad_byte_vector(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() % 4 != 0 {
        vec.push(0); // pad to multiple of four bytes
    }
    vec
}

fn export(model: Model, output: Output) -> Result<(), Error> {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut textures = Vec::new();
    let mut images = Vec::new();
    let skin = model.skin_tables().next().unwrap();

    let loader = Loader::new()?;

    let mesh = push_model(&mut buffer, &mut views, &mut accessors, &model, &skin);

    let materials = model
        .textures()
        .iter()
        .map(|tex| load_material_fallback(&tex.name, &tex.search_paths, &loader))
        .map(|material| {
            push_material(
                &mut buffer,
                &mut views,
                &mut textures,
                &mut images,
                material,
            )
        })
        .collect();

    let node = json::Node {
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: Some(Index::new(0)),
        name: None,
        rotation: None,
        scale: None,
        translation: None,
        skin: None,
        weights: None,
    };

    let g_buffer = json::Buffer {
        byte_length: buffer.len() as u32,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: if output == Output::Standard {
            Some("buffer0.bin".into())
        } else {
            None
        },
    };

    let root = json::Root {
        accessors,
        buffers: vec![g_buffer],
        buffer_views: views,
        meshes: vec![mesh],
        nodes: vec![node],
        scenes: vec![json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: vec![Index::new(0)],
        }],
        materials,
        images,
        textures,
        ..Default::default()
    };

    match output {
        Output::Standard => {
            let _ = fs::create_dir("triangle");

            let writer = fs::File::create("triangle/triangle.gltf").expect("I/O error");
            json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");

            let bin = pad_byte_vector(buffer);
            let mut writer = fs::File::create("triangle/buffer0.bin").expect("I/O error");
            writer.write_all(&bin).expect("I/O error");
        }
        Output::Binary => {
            let json_string = json::serialize::to_string(&root).expect("Serialization error");
            let mut json_offset = json_string.len() as u32;
            align_to_multiple_of_four(&mut json_offset);
            let glb = gltf::binary::Glb {
                header: gltf::binary::Header {
                    magic: *b"glTF",
                    version: 2,
                    length: json_offset + buffer.len() as u32,
                },
                bin: Some(Cow::Owned(pad_byte_vector(buffer))),
                json: Cow::Owned(json_string.into_bytes()),
            };
            let writer = std::fs::File::create("output.glb").expect("I/O error");
            glb.to_writer(writer).expect("glTF binary output error");
        }
    }
    Ok(())
}

fn load(path: &Path) -> Result<Model, vmdl::ModelError> {
    let data = fs::read(path)?;
    let mdl = Mdl::read(&data)?;
    let data = fs::read(path.with_extension("dx90.vtx"))?;
    let vtx = Vtx::read(&data)?;
    let data = fs::read(path.with_extension("vvd"))?;
    let vvd = Vvd::read(&data)?;

    Ok(Model::from_parts(mdl, vtx, vvd))
}

fn main() -> Result<(), Error> {
    let path = PathBuf::from(args_os().nth(1).expect("No model file provided"));
    let source_model = load(&path)?;

    export(source_model, Output::Binary)?;
    Ok(())
}
