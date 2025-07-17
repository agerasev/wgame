use alloc::{
    format,
    string::{String, ToString},
};
use core::{
    fmt::{self, Display},
    ops::RangeInclusive,
};

use anyhow::{Result, bail};
use serde::Serialize;
use smallvec::SmallVec;

#[derive(Clone, Debug, Serialize)]
pub struct BindingInfo {
    pub name: String,
    pub ty: BindingType,
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
                if Self::check_dim(*m)? == Self::check_dim(*n)? {
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
        $crate::binding::BindingType {
            item: $crate::binding::ScalarType::$item,
            dims: [].into_iter().collect(),
        }
    };
    ($item:ident, $n:expr) => {
        $crate::binding::BindingType {
            item: $crate::binding::ScalarType::$item,
            dims: [$n].into_iter().collect(),
        }
    };
    ($item:ident, $m:expr, $n:expr) => {
        $crate::binding::BindingType {
            item: $crate::binding::ScalarType::$item,
            dims: [$m, $n].into_iter().collect(),
        }
    };
}

pub use binding_type;
