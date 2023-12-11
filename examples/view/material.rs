use crate::loader::Loader;
use crate::Error;
use steamy_vdf::{Entry, Table};
use three_d::{Color, CpuMaterial, CpuTexture, TextureData};
use tracing::error;
use vtf::vtf::VTF;

pub fn load_material_fallback(name: &str, search_dirs: &[String], loader: &Loader) -> CpuMaterial {
    match load_material(name, search_dirs, loader) {
        Ok(material) => material,
        Err(e) => {
            error!(
                material = name,
                error = ?e,
                "failed to load material, falling back"
            );
            CpuMaterial {
                albedo: Color {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 255,
                },
                name: name.into(),
                ..Default::default()
            }
        }
    }
}

pub fn load_material(
    name: &str,
    search_dirs: &[String],
    loader: &Loader,
) -> Result<CpuMaterial, Error> {
    let dirs = search_dirs
        .iter()
        .map(|dir| {
            format!(
                "materials/{}",
                dir.to_ascii_lowercase().trim_start_matches("/")
            )
        })
        .collect::<Vec<_>>();
    let path = format!("{}.vmt", name.to_ascii_lowercase().trim_end_matches(".vmt"));
    let raw = loader.load_from_paths(&path, &dirs)?;

    let vmt = parse_vdf(raw)?;
    let vmt = resolve_vmt_patch(vmt, loader)?;

    let material_type = vmt
        .keys()
        .next()
        .ok_or(Error::Other("empty vmt"))?
        .to_ascii_lowercase();
    if material_type == "water" {
        return Ok(CpuMaterial {
            albedo: Color {
                r: 82,
                g: 180,
                b: 217,
                a: 128,
            },
            name: name.into(),
            ..Default::default()
        });
    }

    let table = vmt
        .values()
        .next()
        .ok_or(Error::Other("empty vmt"))?
        .as_table()
        .ok_or(Error::Other("vmt not a table"))?;
    let base_texture = table
        .iter()
        .find_map(|(key, value)| (key.to_ascii_lowercase() == "$basetexture").then_some(value))
        .or_else(|| {
            table
                .iter()
                .find_map(|(key, value)| (key.to_ascii_lowercase() == "eyes_dx8").then_some(value))
                .and_then(|eyes| eyes.as_table())
                .and_then(|eyes| {
                    eyes.iter().find_map(|(key, value)| {
                        (key.to_ascii_lowercase() == "$basetexture").then_some(value)
                    })
                })
        })
        .ok_or(Error::Other("no $basetexture"))?
        .as_value()
        .ok_or(Error::Other("$basetexture not a value"))?
        .to_string()
        .to_ascii_lowercase()
        .replace('\\', "/")
        .replace('\t', "/t");
    let texture = load_texture(base_texture.as_str(), loader)?;
    Ok(CpuMaterial {
        name: name.into(),
        albedo: Color::WHITE,
        albedo_texture: Some(texture),
        ..CpuMaterial::default()
    })
}

fn parse_vdf(bytes: Vec<u8>) -> Result<Table, Error> {
    let mut reader = steamy_vdf::Reader::from(bytes.as_slice());
    Table::load(&mut reader).map_err(|e| {
        println!("{}", String::from_utf8_lossy(&bytes));
        error!(
            source = String::from_utf8_lossy(&bytes).to_string(),
            error = ?e,
            "failed to parse vmt"
        );
        Error::Other("failed to parse vdf")
    })
}

fn load_texture(name: &str, loader: &Loader) -> Result<CpuTexture, Error> {
    let path = format!(
        "materials/{}.vtf",
        name.trim_end_matches(".vtf").trim_start_matches("/")
    );
    let mut raw = loader.load(&path)?;
    let vtf = VTF::read(&mut raw)?;
    let image = vtf.highres_image.decode(0)?;
    Ok(CpuTexture {
        name: name.into(),
        data: TextureData::RgbaU8(image.into_rgba8().pixels().map(|pixel| pixel.0).collect()),
        height: vtf.header.height as u32,
        width: vtf.header.width as u32,
        ..CpuTexture::default()
    })
}

fn resolve_vmt_patch(vmt: Table, loader: &Loader) -> Result<Table, Error> {
    if vmt.len() != 1 {
        panic!("vmt with more than 1 item?");
    }
    if let Some(Entry::Table(patch)) = vmt.get("patch") {
        let include = patch
            .get("include")
            .expect("no include in patch")
            .as_value()
            .expect("include is not a value")
            .to_string();
        let _replace = patch
            .get("replace")
            .expect("no replace in patch")
            .as_table()
            .expect("replace is not a table");
        let included_raw = loader.load(&include.to_ascii_lowercase())?;

        // todo actually patch
        parse_vdf(included_raw)
    } else {
        Ok(vmt)
    }
}
