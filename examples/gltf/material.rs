use crate::loader::Loader;
use crate::Error;
use image::DynamicImage;
use std::str::FromStr;
use steamy_vdf::{Entry, Table};
use tracing::error;
use vtf::vtf::VTF;

fn get_path(vmt: &Entry, name: &str) -> Option<String> {
    Some(vmt.lookup(name)?.as_str()?.replace('\\', "/"))
}

pub fn load_material_fallback(name: &str, search_dirs: &[String], loader: &Loader) -> MaterialData {
    match load_material(name, search_dirs, loader) {
        Ok(mat) => mat,
        Err(e) => {
            error!(error = ?e, "failed to load material");
            MaterialData {
                name: name.into(),
                color: [255, 0, 255, 255],
                ..MaterialData::default()
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct MaterialData {
    pub name: String,
    pub color: [u8; 4],
    pub texture: Option<DynamicImage>,
    pub alpha_test: Option<f32>,
    pub bump_map: Option<DynamicImage>,
    pub translucent: bool,
}

pub fn load_material(
    name: &str,
    search_dirs: &[String],
    loader: &Loader,
) -> Result<MaterialData, Error> {
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

    Ok(MaterialData {
        color: [255; 4],
        name: name.into(),
        texture: Some(texture),
        bump_map,
        alpha_test: alpha_test.then_some(alpha_cutout),
        translucent: translucent | glass | alpha_test,
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

fn load_texture(name: &str, loader: &Loader, _alpha: bool) -> Result<DynamicImage, Error> {
    let path = format!(
        "materials/{}.vtf",
        name.trim_end_matches(".vtf").trim_start_matches('/')
    );
    let mut raw = loader.load(&path)?;
    let vtf = VTF::read(&mut raw)?;
    let image = vtf.highres_image.decode(0)?;
    Ok(image)
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
