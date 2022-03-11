use crate::mdl::Mdl;
use std::ops::Deref;

/// A handle represents a mdl structure in the mdl file and the mdl file containing it.
///
/// Keeping a reference of the mdl file with the mdl is required since a lot of mdl types
/// reference parts from other structures in the mdl file
#[derive(Debug)]
pub struct Handle<'a, T> {
    _mdl: &'a Mdl,
    data: &'a T,
}

impl<T> Clone for Handle<'_, T> {
    fn clone(&self) -> Self {
        Handle { ..*self }
    }
}

impl<'a, T> AsRef<T> for Handle<'a, T> {
    fn as_ref(&self) -> &'a T {
        self.data
    }
}

impl<T> Deref for Handle<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
