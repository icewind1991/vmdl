use crate::loader::LoadError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Three(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    Mdl(#[from] vmdl::ModelError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Loader(#[from] LoadError),
    #[error(transparent)]
    Vtf(#[from] vtf::Error),
    #[error("{0}")]
    Other(&'static str),
    #[error("Skin index out of bounds: {0}, model only has {1} skins")]
    SkinOutOfBounds(u16, u16),
}
