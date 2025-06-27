use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    ops::Deref,
};

use anyhow::Result;
use fxhash::hash64;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ShaderSource<'a> {
    source: Cow<'a, str>,
    hash: u64,
}

impl<'a> ShaderSource<'a> {
    pub fn new(source: impl Into<Cow<'a, str>>) -> Self {
        let source = source.into();
        let hash = hash64(&source);
        Self { source, hash }
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

impl<'a> Hash for ShaderSource<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

/// Signed distance function
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sdf {
    expr: Cow<'static, str>,
}

impl Sdf {
    pub fn new(expr: impl Into<Cow<'static, str>>) -> Result<Self> {
        Ok(Self { expr: expr.into() })
    }
}

impl Deref for Sdf {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}
