use crate::mdl::{Bone, BoneId, Mdl};
use std::collections::VecDeque;
use std::ops::Deref;

/// A handle represents a mdl structure in the mdl file and the mdl file containing it.
///
/// Keeping a reference of the mdl file with the mdl is required since a lot of mdl types
/// reference parts from other structures in the mdl file
#[derive(Debug, Clone)]
pub struct Handle<'a, T, K> {
    mdl: &'a Mdl,
    data: &'a T,
    key: K,
}

impl<T, K: PartialEq> PartialEq for Handle<'_, T, K> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<'a, T, K> Handle<'a, T, K> {
    pub fn new(mdl: &'a Mdl, data: &'a T, key: K) -> Self {
        Self { mdl, data, key }
    }
}

impl<T, K: Clone> Handle<'_, T, K> {
    pub fn key(&self) -> K {
        self.key.clone()
    }
}

impl<'a, T, K> AsRef<T> for Handle<'a, T, K> {
    fn as_ref(&self) -> &'a T {
        self.data
    }
}

impl<T, K> Deref for Handle<'_, T, K> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> Handle<'a, Bone, BoneId> {
    pub fn parent(&self) -> Option<Self> {
        Some(Self::new(
            self.mdl,
            self.mdl.bones.get(usize::from(self.parent))?,
            self.parent,
        ))
    }

    pub fn children(&self) -> impl Iterator<Item = Self> + 'a {
        let key = self.key();
        let mdl = self.mdl;
        self.mdl
            .bones
            .iter()
            .enumerate()
            .filter(move |(_, bone)| bone.parent == key)
            .map(move |(i, bone)| Self::new(mdl, bone, i.into()))
    }

    pub fn tree(&self) -> impl Iterator<Item = Self> {
        BoneTreeIter::new(self.clone())
    }

    pub fn ancestors(&self) -> impl Iterator<Item = Self> {
        BoneAncestorsIter { bone: self.clone() }
    }

    pub fn is_affected_by(&self, bone_id: BoneId) -> bool {
        self.key == bone_id || self.ancestors().any(|ancestor| ancestor.key == bone_id)
    }
}

struct BoneTreeIter<'a> {
    queue: VecDeque<Handle<'a, Bone, BoneId>>,
}

impl<'a> BoneTreeIter<'a> {
    pub fn new(root: Handle<'a, Bone, BoneId>) -> Self {
        let mut queue = VecDeque::with_capacity(16);
        queue.push_back(root);
        BoneTreeIter { queue }
    }
}

impl<'a> Iterator for BoneTreeIter<'a> {
    type Item = Handle<'a, Bone, BoneId>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.queue.pop_front()?;
        self.queue.extend(next.children());

        Some(next)
    }
}

struct BoneAncestorsIter<'a> {
    bone: Handle<'a, Bone, BoneId>,
}

impl<'a> Iterator for BoneAncestorsIter<'a> {
    type Item = Handle<'a, Bone, BoneId>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bone.parent()?;
        self.bone = next.clone();
        Some(next)
    }
}
