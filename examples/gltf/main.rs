mod convert;
mod error;
mod loader;
mod material;

use gltf_json as json;

use std::fs;

use crate::convert::{push_material, push_model};
use crate::loader::Loader;
use crate::material::load_material_fallback;
use clap::Parser;
pub use error::Error;
use gltf_json::Index;
use main_error::MainResult;
use std::borrow::Cow;
use std::path::PathBuf;
use vmdl::Model;

fn align_to_multiple_of_four(n: &mut u32) {
    *n = (*n + 3) & !3;
}

fn pad_byte_vector(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() % 4 != 0 {
        vec.push(0); // pad to multiple of four bytes
    }
    vec
}

fn export(model: Model, skin: u16, target: PathBuf) -> Result<(), Error> {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut textures = Vec::new();
    let mut images = Vec::new();

    let skin = model
        .skin_tables()
        .nth(skin as usize)
        .ok_or_else(|| Error::SkinOutOfBounds(skin, model.skin_tables().count() as u16))?;

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
        uri: None,
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
    let writer = fs::File::create(target).expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    source: PathBuf,
    target: PathBuf,

    #[arg(short, long, default_value_t = 0)]
    skin: u16,
}

fn main() -> MainResult {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let source_model = Model::from_path(&args.source)?;

    export(source_model, args.skin, args.target)?;
    Ok(())
}
