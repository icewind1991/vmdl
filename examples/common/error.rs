use miette::Diagnostic;
use std::string::FromUtf8Error;
use tf_asset_loader::LoaderError;
use thiserror::Error;
use vmt_parser::VdfError;

#[allow(dead_code)]
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    Three(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    Mdl(#[from] vmdl::ModelError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Loader(#[from] LoaderError),
    #[error(transparent)]
    Vtf(#[from] vtf::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Vdf(#[from] VdfError),
    #[error("{0}")]
    Other(String),
    #[error("Skin index out of bounds: {0}, model only has {1} skins")]
    SkinOutOfBounds(u16, u16),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
}
