use std::{
    fmt::{self, Display},
    mem::replace,
    ops::RangeInclusive,
};

use anyhow::{Result, bail};
use serde::Serialize;
use smallvec::SmallVec;

#[derive(Clone, Default, Debug, Serialize)]
pub struct BindingList(SmallVec<[Binding; 2]>);

impl FromIterator<Binding> for BindingList {
    fn from_iter<T: IntoIterator<Item = Binding>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for BindingList {
    type Item = Binding;
    type IntoIter = <SmallVec<[Binding; 2]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BindingList {
    type Item = &'a Binding;
    type IntoIter = <&'a SmallVec<[Binding; 2]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl BindingList {
    pub fn iter(&self) -> impl Iterator<Item = &Binding> + '_ {
        self.into_iter()
    }

    pub fn push(&mut self, item: Binding) {
        self.0.push(item);
    }

    pub fn chain(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        Self(self.0)
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        for Binding { name, .. } in self.0.iter_mut() {
            *name = if name.is_empty() {
                prefix.to_string()
            } else {
                format!("{prefix}_{name}")
            };
        }
        self
    }

    pub fn size(&self) -> u64 {
        self.0.iter().map(|Binding { ty, .. }| ty.size()).sum()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn count(&self) -> u32 {
        self.len() as u32
    }

    pub fn layout(&self, start_location: u32) -> Result<Vec<wgpu::VertexAttribute>> {
        self.0
            .iter()
            .scan(
                (start_location, 0),
                |(index, offset), Binding { name, ty }| {
                    Some(Ok(wgpu::VertexAttribute {
                        shader_location: replace(index, *index + 1),
                        offset: replace(offset, *offset + ty.size()),
                        format: match ty.to_attribute() {
                            Ok(a) => a,
                            Err(e) => {
                                return Some(Err(e.context(format!(
                                    "Error getting attribute '{name}' of type {ty:?}",
                                ))));
                            }
                        },
                    }))
                },
            )
            .collect()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Binding {
    pub name: String,
    pub ty: BindingType,
}

impl Binding {
    pub fn new(name: impl Into<String>, ty: BindingType) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BindingType {
    pub item: ScalarType,
    pub dims: SmallVec<[usize; 2]>,
}

impl BindingType {
    const DIM_RANGE: RangeInclusive<usize> = 2..=4;

    fn check_dim(n: usize) -> Result<usize, anyhow::Error> {
        if !Self::DIM_RANGE.contains(&n) {
            bail!("Dimension must be in range {:?}, got {n}", Self::DIM_RANGE)
        }
        Ok(n)
    }

    pub fn to_wgsl(&self) -> Result<String> {
        Ok(match self.dims.as_slice() {
            [] => self.item.to_string(),
            [n] => format!("vec{}<{}>", Self::check_dim(*n)?, self.item),
            [m, n] => {
                if Self::check_dim(*m)? != Self::check_dim(*n)? {
                    format!("mat{m}x{n}<{}>", self.item)
                } else {
                    format!("mat{m}<{}>", self.item)
                }
            }
            other => bail!("WGSL supports up to 2 dimensions, got {}", other.len()),
        })
    }

    pub fn to_attribute(&self) -> Result<wgpu::VertexFormat> {
        Ok(match self.dims.as_slice() {
            [] => match self.item {
                ScalarType::U8 => wgpu::VertexFormat::Uint8,
                ScalarType::U16 => wgpu::VertexFormat::Uint16,
                ScalarType::U32 => wgpu::VertexFormat::Uint32,
                ScalarType::I8 => wgpu::VertexFormat::Sint8,
                ScalarType::I16 => wgpu::VertexFormat::Sint16,
                ScalarType::I32 => wgpu::VertexFormat::Sint32,
                ScalarType::U64 | ScalarType::I64 => {
                    bail!("No support for 64-bit integer in attributes")
                }
                ScalarType::F16 => wgpu::VertexFormat::Float16,
                ScalarType::F32 => wgpu::VertexFormat::Float32,
                ScalarType::F64 => wgpu::VertexFormat::Float64,
            },
            [n] => match (self.item, Self::check_dim(*n)?) {
                (ScalarType::U8, 2) => wgpu::VertexFormat::Uint8x2,
                (ScalarType::U8, 4) => wgpu::VertexFormat::Uint8x4,
                (ScalarType::U16, 2) => wgpu::VertexFormat::Uint16x2,
                (ScalarType::U16, 4) => wgpu::VertexFormat::Uint16x4,
                (ScalarType::U32, 2) => wgpu::VertexFormat::Uint32x2,
                (ScalarType::U32, 3) => wgpu::VertexFormat::Uint32x3,
                (ScalarType::U32, 4) => wgpu::VertexFormat::Uint32x4,
                (ScalarType::I8, 2) => wgpu::VertexFormat::Sint8x2,
                (ScalarType::I8, 4) => wgpu::VertexFormat::Sint8x4,
                (ScalarType::I16, 2) => wgpu::VertexFormat::Sint16x2,
                (ScalarType::I16, 4) => wgpu::VertexFormat::Sint16x4,
                (ScalarType::I32, 2) => wgpu::VertexFormat::Sint32x2,
                (ScalarType::I32, 3) => wgpu::VertexFormat::Sint32x3,
                (ScalarType::I32, 4) => wgpu::VertexFormat::Sint32x4,
                (ScalarType::U64 | ScalarType::I64, _) => {
                    bail!("No support for 64-bit integer in attributes")
                }
                (ScalarType::F16, 2) => wgpu::VertexFormat::Float16x2,
                (ScalarType::F16, 4) => wgpu::VertexFormat::Float16x4,
                (ScalarType::F32, 2) => wgpu::VertexFormat::Float32x2,
                (ScalarType::F32, 3) => wgpu::VertexFormat::Float32x3,
                (ScalarType::F32, 4) => wgpu::VertexFormat::Float32x4,
                (ScalarType::F64, 2) => wgpu::VertexFormat::Float64x2,
                (ScalarType::F64, 3) => wgpu::VertexFormat::Float64x3,
                (ScalarType::F64, 4) => wgpu::VertexFormat::Float64x4,
                (ty, n) => {
                    bail!("No attribute type for {n}-dimensional vector of {ty}")
                }
            },
            other => bail!(
                "Attribute does not support 2 or more dimensions, got {} dims",
                other.len()
            ),
        })
    }

    pub fn size(&self) -> u64 {
        self.item.size() * self.dims.iter().product::<usize>() as u64
    }
}

impl Serialize for BindingType {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &self
                .to_wgsl()
                .map_err(|err| serde::ser::Error::custom(err.to_string()))?,
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(into = "String")]
pub enum ScalarType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F16,
    F32,
    F64,
}

impl ScalarType {
    pub fn size(&self) -> u64 {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 => 4,
            Self::U64 | Self::I64 => 8,
            Self::F16 => 2,
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::I16 => "i16",
            Self::U32 => "u32",
            Self::I32 => "i32",
            Self::U64 => "u64",
            Self::I64 => "i64",
            Self::F16 => "f16",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<ScalarType> for String {
    fn from(value: ScalarType) -> Self {
        value.as_str().to_string()
    }
}

#[macro_export]
macro_rules! binding_type {
    ($item:ident) => {
        $crate::BindingType {
            item: $crate::ScalarType::$item,
            dims: [].into_iter().collect(),
        }
    };
    ($item:ident, $n:expr) => {
        $crate::BindingType {
            item: $crate::ScalarType::$item,
            dims: [$n].into_iter().collect(),
        }
    };
    ($item:ident, $m:expr, $n:expr) => {
        $crate::BindingType {
            item: $crate::ScalarType::$item,
            dims: [$m, $n].into_iter().collect(),
        }
    };
}
pub use binding_type;
