#[path = "../common/error.rs"]
mod error;
#[path = "../common/materials.rs"]
mod material;

use crate::error::Error;
use crate::material::{load_material_fallback, MaterialData};
use cgmath::{vec3, Matrix4, SquareMatrix};
use std::collections::HashMap;
use std::env::args_os;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tf_asset_loader::Loader;
use three_d::{
    AmbientLight, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuModel,
    CpuTexture, DepthMaterial, DirectionalLight, FrameOutput, Light, NormalMaterial, ORMMaterial,
    OrbitControl, PhysicalMaterial, PositionMaterial, UVMaterial, Vec2, Vec4, Window,
    WindowSettings,
};
use three_d_asset::{
    degrees, Geometry, Mat4, Positions, Primitive, Srgba, TextureData, Vec3, Viewport,
};
use vmdl::Model;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum DebugType {
    POSITION,
    NORMAL,
    COLOR,
    DEPTH,
    ORM,
    UV,
    NONE,
}

fn main() -> Result<(), Error> {
    miette::set_panic_hook();
    tracing_subscriber::fmt::init();

    let mut args = args_os();
    let _ = args.next();
    let path = PathBuf::from(args.next().expect("No demo file provided"));
    let source_model = Model::from_path(&path)?;

    let window = Window::new(WindowSettings {
        title: path.display().to_string(),
        min_size: (512, 512),
        max_size: Some((1920, 1080)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(2.0, 2.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(90.0),
        0.01,
        300.0,
    );

    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);
    let mut gui = three_d::GUI::new(&context);

    let loader = Loader::new().expect("loader");
    let skin_count = source_model.skin_tables().count();
    let animation_count = source_model.animations().count();

    let ph_material = PhysicalMaterial {
        albedo: Srgba {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        },
        ..Default::default()
    };

    let mut directional = [
        DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(1.0, -1.0, 0.0)),
        DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(1.0, 1.0, 0.0)),
    ];
    let mut ambient = AmbientLight {
        color: Srgba::WHITE,
        intensity: 0.2,
        ..Default::default()
    };

    // main loop
    let mut shadows_enabled = true;
    let mut directional_intensity = directional[0].intensity;
    let mut depth_max = 30.0;
    let mut fov = 60.0;
    let mut debug_type = DebugType::NONE;
    let mut skin_index = 0;
    let mut animation_index = 0;
    let mut frame = 0;
    let mut skip_count = 0;
    let mut playing = false;

    let model_builder = ModelBuilder::new(source_model, loader);

    window.render_loop(move |mut frame_input| {
        let model_key = ModelKey::new(skin_index, animation_index, frame);

        let model = model_builder.get(&context, model_key);
        let frame_count = model_builder.frame_count(animation_index);

        let mut change = frame_input.first_frame;
        let mut panel_width = frame_input.viewport.width;
        change |= gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");

                    ui.label("Light options");
                    ui.add(
                        Slider::new(&mut ambient.intensity, 0.0..=1.0).text("Ambient intensity"),
                    );
                    ui.add(
                        Slider::new(&mut directional_intensity, 0.0..=1.0)
                            .text("Directional intensity"),
                    );
                    directional[0].intensity = directional_intensity;
                    directional[1].intensity = directional_intensity;
                    if ui.checkbox(&mut shadows_enabled, "Shadows").clicked() && !shadows_enabled {
                        directional[0].clear_shadow_map();
                        directional[1].clear_shadow_map();
                    }

                    ui.label("Debug options");
                    ui.radio_value(&mut debug_type, DebugType::NONE, "None");
                    ui.radio_value(&mut debug_type, DebugType::POSITION, "Position");
                    ui.radio_value(&mut debug_type, DebugType::NORMAL, "Normal");
                    ui.radio_value(&mut debug_type, DebugType::COLOR, "Color");
                    ui.radio_value(&mut debug_type, DebugType::DEPTH, "Depth");
                    ui.radio_value(&mut debug_type, DebugType::ORM, "ORM");
                    ui.radio_value(&mut debug_type, DebugType::UV, "UV");

                    ui.label("View options");
                    ui.add(Slider::new(&mut skin_index, 0..=(skin_count - 1)).text("Skin"));
                    ui.add(Slider::new(&mut depth_max, 1.0..=30.0).text("Depth max"));
                    ui.add(Slider::new(&mut fov, 45.0..=90.0).text("FOV"));

                    ui.label("Animation");
                    ui.add(
                        Slider::new(&mut animation_index, 0..=(animation_count - 1))
                            .text("Animation"),
                    );
                    ui.add(Slider::new(&mut frame, 0..=(frame_count - 1)).text("Frame"));
                    ui.add(Checkbox::new(&mut playing, "Play"));

                    ui.label("Position");
                    ui.add(Label::new(format!("\tx: {}", camera.position().x)));
                    ui.add(Label::new(format!("\ty: {}", camera.position().y)));
                    ui.add(Label::new(format!("\tz: {}", camera.position().z)));
                });
                panel_width = gui_context.used_size().x as u32;
            },
        );
        if playing {
            skip_count += 1;
            if skip_count > 4 {
                frame += 1;
                if frame >= frame_count {
                    frame = 0;
                }
                change = true;
                skip_count = 0;
            }
        }

        let viewport = Viewport {
            x: panel_width as i32,
            y: 0,
            width: frame_input.viewport.width - panel_width,
            height: frame_input.viewport.height,
        };
        change |= camera.set_viewport(viewport);
        change |= control.handle_events(&mut camera, &mut frame_input.events);

        // Draw
        {
            let z_near = camera.z_near();
            let z_far = camera.z_far();
            camera.set_perspective_projection(degrees(fov), z_near, z_far);
            if shadows_enabled {
                directional[0].generate_shadow_map(1024, model.iter().map(|gm| &gm.geometry));
                directional[1].generate_shadow_map(1024, model.iter().map(|gm| &gm.geometry));
            }

            let lights = &[&ambient as &dyn Light, &directional[0], &directional[1]];

            // Light pass
            let screen = frame_input.screen();
            let target = screen.clear(ClearState::default());
            match debug_type {
                DebugType::NORMAL => target.render_with_material(
                    &NormalMaterial::from_physical_material(&ph_material),
                    &camera,
                    model.iter().map(|gm| &gm.geometry),
                    lights,
                ),
                DebugType::DEPTH => {
                    let depth_material = DepthMaterial {
                        max_distance: Some(depth_max),
                        ..DepthMaterial::default()
                    };
                    target.render_with_material(
                        &depth_material,
                        &camera,
                        model.iter().map(|gm| &gm.geometry),
                        lights,
                    )
                }
                DebugType::ORM => target.render_with_material(
                    &ORMMaterial::from_physical_material(&ph_material),
                    &camera,
                    model.iter().map(|gm| &gm.geometry),
                    lights,
                ),
                DebugType::POSITION => {
                    let position_material = PositionMaterial::default();
                    target.render_with_material(
                        &position_material,
                        &camera,
                        model.iter().map(|gm| &gm.geometry),
                        lights,
                    )
                }
                DebugType::UV => {
                    let uv_material = UVMaterial::default();
                    target.render_with_material(
                        &uv_material,
                        &camera,
                        model.iter().map(|gm| &gm.geometry),
                        lights,
                    )
                }
                DebugType::COLOR => target.render_with_material(
                    &ColorMaterial::from_physical_material(&ph_material),
                    &camera,
                    model.iter().map(|gm| &gm.geometry),
                    lights,
                ),
                DebugType::NONE => target.render(&camera, model.as_ref(), lights),
            }
            .write(|| gui.render())
            .expect("failed to render");
        }

        let _ = change;

        FrameOutput::default()
    });
    Ok(())
}

// 1 hammer unit is ~1.905cm
const UNIT_SCALE: f32 = 1.0 / (1.905 * 100.0);

pub fn map_coords<C: Into<Vec3>>(vec: C) -> Vec3 {
    let vec = vec.into();
    Vec3 {
        x: vec.y * UNIT_SCALE,
        y: vec.z * UNIT_SCALE,
        z: vec.x * UNIT_SCALE,
    }
}

fn model_to_model(
    model: &Model,
    loader: &Loader,
    skin: usize,
    animation: usize,
    frame: usize,
) -> CpuModel {
    let skin = model.skin_tables().nth(skin).unwrap();

    let transforms = Matrix4::identity();

    let animation = model
        .animations()
        .nth(animation)
        .unwrap_or_else(|| model.animations().next().unwrap());

    let geometries = model
        .meshes()
        .map(|mesh| {
            let positions: Vec<Vec3> = mesh
                .vertices()
                .map(|vertex| model.apply_animation(animation, vertex, frame))
                .map(|position| map_coords(position) * 10.0)
                .map(|vertex: Vec3| (transforms * vertex.extend(1.0)).truncate())
                .collect();
            let normals: Vec<Vec3> = mesh.vertices().map(|vertex| vertex.normal.into()).collect();
            let uvs: Vec<Vec2> = mesh
                .vertices()
                .map(|vertex| vertex.texture_coordinates.into())
                .collect();
            let tangents: Vec<Vec4> = mesh.tangents().map(Vec4::from).collect();

            let triangles = CpuMesh {
                positions: Positions::F32(positions),
                normals: Some(normals),
                uvs: Some(uvs),
                tangents: Some(tangents),
                ..Default::default()
            };
            Primitive {
                name: "model".into(),
                geometry: Geometry::Triangles(triangles),
                transformation: Mat4::identity(),
                animations: vec![],
                material_index: skin.texture_index(mesh.material_index()),
            }
        })
        .collect();

    let materials = model
        .textures()
        .iter()
        .map(|texture| load_material_fallback(&texture.name, model.texture_directories(), loader))
        .map(convert_material)
        .collect();

    CpuModel {
        name: "model".into(),
        materials,
        geometries,
    }
}

fn convert_material(material: MaterialData) -> CpuMaterial {
    CpuMaterial {
        albedo: Srgba::new(
            material.color[0],
            material.color[1],
            material.color[2],
            material.color[3],
        ),
        name: material.name,
        albedo_texture: material
            .texture
            .map(|tex| convert_texture(tex, material.translucent | material.alpha_test.is_some())),
        alpha_cutout: material.alpha_test,
        normal_texture: material.bump_map.map(|tex| convert_texture(tex, true)),
        ..CpuMaterial::default()
    }
}
fn convert_texture(texture: material::TextureData, keep_alpha: bool) -> CpuTexture {
    let width = texture.image.width();
    let height = texture.image.height();
    let data = if keep_alpha {
        TextureData::RgbaU8(
            texture
                .image
                .into_rgba8()
                .pixels()
                .map(|pixel| pixel.0)
                .collect(),
        )
    } else {
        TextureData::RgbU8(
            texture
                .image
                .into_rgb8()
                .pixels()
                .map(|pixel| pixel.0)
                .collect(),
        )
    };
    CpuTexture {
        data,
        name: texture.name,
        height,
        width,
        ..CpuTexture::default()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct ModelKey {
    skin: usize,
    animation: usize,
    frame: usize,
}

impl ModelKey {
    pub fn new(skin: usize, animation: usize, frame: usize) -> Self {
        ModelKey {
            skin,
            animation,
            frame,
        }
    }
}

pub struct ModelBuilder {
    source: Model,
    loader: Loader,
    cache: Mutex<HashMap<ModelKey, Arc<three_d::Model<PhysicalMaterial>>>>,
}

impl ModelBuilder {
    fn new(source: Model, loader: Loader) -> Self {
        ModelBuilder {
            source,
            loader,
            cache: Mutex::default(),
        }
    }

    fn frame_count(&self, animation: usize) -> usize {
        self.source
            .animations()
            .nth(animation)
            .map(|animation| animation.frame_count)
            .unwrap_or_default()
    }

    fn get(&self, context: &Context, key: ModelKey) -> Arc<three_d::Model<PhysicalMaterial>> {
        self.cache
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(|| {
                let cpu_model = model_to_model(
                    &self.source,
                    &self.loader,
                    key.skin,
                    key.animation,
                    key.frame,
                );
                Arc::new(
                    three_d::Model::new(context, &cpu_model).expect("failed to build gpu model"),
                )
            })
            .clone()
    }
}
