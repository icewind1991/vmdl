use crate::loader::Loader;
use crate::Error;
use std::str::FromStr;
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

fn get_path(vmt: &Entry, name: &str) -> Option<String> {
    Some(vmt.lookup(name)?.as_str()?.replace('\\', "/"))
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

    let vmt = parse_vdf(&raw)?;
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
        .cloned()
        .ok_or(Error::Other("empty vmt"))?;
    let base_texture = get_path(&table, "$basetexture").ok_or(Error::Other("no $basetexture"))?;

    let translucent = table
        .lookup("$translucent")
        .map(|val| val.as_str() == Some("1"))
        .unwrap_or_default();
    let glass = table
        .lookup("$surfaceprop")
        .map(|val| val.as_str() == Some("glass"))
        .unwrap_or_default();
    let alpha_test = table
        .lookup("$alphatest")
        .map(|val| val.as_str() == Some("1"))
        .unwrap_or_default();
    let texture = load_texture(
        base_texture.as_str(),
        loader,
        translucent | glass | alpha_test,
    )?;

    let alpha_cutout = table
        .lookup("$alphatestreference")
        .and_then(Entry::as_str)
        .and_then(|val| f32::from_str(val).ok())
        .unwrap_or(1.0);

    let bump_map = get_path(&table, "$bumpmap")
        .map(|path| load_texture(&path, loader, true).ok())
        .flatten();

    Ok(CpuMaterial {
        name: name.into(),
        albedo: Color::WHITE,
        albedo_texture: Some(texture),
        alpha_cutout: alpha_test.then_some(alpha_cutout),
        normal_texture: bump_map,
        ..CpuMaterial::default()
    })
}

fn parse_vdf(bytes: &[u8]) -> Result<Table, Error> {
    let bytes = bytes.to_ascii_lowercase();
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

fn load_texture(name: &str, loader: &Loader, alpha: bool) -> Result<CpuTexture, Error> {
    let path = format!(
        "materials/{}.vtf",
        name.trim_end_matches(".vtf").trim_start_matches('/')
    );
    let mut raw = loader.load(&path)?;
    let vtf = VTF::read(&mut raw)?;
    let image = vtf.highres_image.decode(0)?;
    let texture_data = if alpha {
        TextureData::RgbaU8(image.into_rgba8().pixels().map(|pixel| pixel.0).collect())
    } else {
        TextureData::RgbU8(image.into_rgb8().pixels().map(|pixel| pixel.0).collect())
    };
    Ok(CpuTexture {
        name: name.into(),
        data: texture_data,
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
        parse_vdf(&included_raw)
    } else {
        Ok(vmt)
    }
}
