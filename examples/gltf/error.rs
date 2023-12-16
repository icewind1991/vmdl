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
}
