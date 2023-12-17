use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::PathBuf;
use steamlocate::SteamDir;
use thiserror::Error;
use tracing::{debug, error, info};
use vpk::VPK;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("{0}")]
    Other(&'static str),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl From<&'static str> for LoadError {
    fn from(e: &'static str) -> Self {
        LoadError::Other(e)
    }
}

pub struct Loader {
    tf_dir: PathBuf,
    download: PathBuf,
    vpks: Vec<VPK>,
}

impl Debug for Loader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Loader")
            .field("tf_dir", &self.tf_dir)
            .finish_non_exhaustive()
    }
}

impl Loader {
    pub fn new() -> Result<Self, LoadError> {
        let tf_dir = SteamDir::locate()
            .ok_or("Can't find steam directory")?
            .app(&440)
            .ok_or("Can't find tf2 directory")?
            .path
            .join("tf");
        let download = tf_dir.join("download");
        let vpks = tf_dir
            .read_dir()?
            .filter_map(|item| item.ok())
            .filter_map(|item| Some(item.path().to_str()?.to_string()))
            .filter(|path| path.ends_with("dir.vpk"))
            .map(|path| vpk::from_path(&path))
            .filter_map(|res| res.ok())
            .collect();

        Ok(Loader {
            tf_dir,
            download,
            vpks,
        })
    }

    #[tracing::instrument]
    pub fn exists(&self, name: &str) -> bool {
        debug!("loading {}", name);
        if name.ends_with("bsp") {
            let path = self.tf_dir.join(name);
            if path.exists() {
                return true;
            }
            let path = self.download.join(name);
            if path.exists() {
                return true;
            }
        }
        for vpk in self.vpks.iter() {
            if vpk.tree.contains_key(name) {
                return true;
            }
        }
        false
    }

    #[tracing::instrument]
    pub fn load(&self, name: &str) -> Result<Vec<u8>, LoadError> {
        debug!("loading {}", name);
        if name.ends_with("bsp") {
            let path = self.tf_dir.join(name);
            if path.exists() {
                debug!("found in tf2 dir");
                return Ok(fs::read(path)?);
            }
            let path = self.download.join(name);
            if path.exists() {
                debug!("found in download dir");
                return Ok(fs::read(path)?);
            }
        }
        for vpk in self.vpks.iter() {
            if let Some(entry) = vpk.tree.get(name) {
                let data = entry.get()?.into_owned();
                debug!("got {} bytes from vpk", data.len());
                return Ok(data);
            }
        }
        info!("Failed to find {} in vpk", name);
        Err(LoadError::Other("Can't find file in vpks"))
    }

    pub fn load_from_paths(&self, name: &str, paths: &[String]) -> Result<Vec<u8>, LoadError> {
        for path in paths {
            if self.exists(&format!("{}{}", path, name)) {
                return self.load(&format!("{}{}", path, name));
            }
        }
        error!("Failed to find {} in vpk paths: {}", name, paths.join(", "));
        Err(LoadError::Other("Can't find file in vpks"))
    }
}
