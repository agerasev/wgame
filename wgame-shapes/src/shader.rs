use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    fmt::{self, Display},
    ops::RangeInclusive,
};

use anyhow::{Result, bail};
use minijinja::{Environment, UndefinedBehavior, Value, value::ValueKind};
use serde::Serialize;

#[derive(Clone, Default, Debug, Serialize)]
pub struct ShaderConfig {
    pub fragment_modifier: String,
    pub instances: Vec<BindingInfo>,
    pub uniforms: Vec<BindingInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BindingInfo {
    pub name: String,
    pub ty: BindingType,
}

#[derive(Clone, Debug)]
pub struct BindingType {
    pub dims: Vec<usize>,
    pub item: ScalarType,
}

impl BindingType {
    const DIM_RANGE: RangeInclusive<usize> = 2..=4;

    fn check_dim(n: usize) -> Result<usize, anyhow::Error> {
        if !Self::DIM_RANGE.contains(&n) {
            bail!("Dimension must be in range {:?}, got {n}", Self::DIM_RANGE)
        }
        Ok(n)
    }

    pub fn to_wgsl(&self) -> Result<String, anyhow::Error> {
        match self.dims.as_slice() {
            [] => Ok(self.item.to_string()),
            [n] => Ok(format!("vec{}<{}>", Self::check_dim(*n)?, self.item)),
            [m, n] => {
                if Self::check_dim(*m)? == Self::check_dim(*n)? {
                    Ok(format!("mat{m}x{n}<{}>", self.item))
                } else {
                    Ok(format!("mat{m}<{}>", self.item))
                }
            }
            other => bail!("Maximum number of dimensions is 2, got {}", other.len()),
        }
    }

    pub fn size(&self) -> usize {
        self.item.size() * self.dims.iter().product::<usize>()
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

#[derive(Clone, Debug, Serialize)]
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
    pub fn size(&self) -> usize {
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

#[derive(Clone, Debug)]
pub struct ShaderSource {
    name: String,
    env: Environment<'static>,
}

impl ShaderSource {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let mut env = Environment::new();

        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_trim_blocks(true);
        env.set_lstrip_blocks(true);
        env.set_keep_trailing_newline(true);

        env.add_filter("add", add);
        env.add_filter("enumerate", enumerate);

        env.add_template_owned(name.clone(), source.into())?;

        Ok(Self { name, env })
    }

    pub fn substitute(&self, ctx: &ShaderConfig) -> Result<String> {
        let template = self.env.get_template(&self.name)?;
        let rendered = template.render(ctx)?;
        Ok(rendered)
    }
}

pub fn add(x: Value, y: Value) -> Result<Value, minijinja::Error> {
    Ok(if let Ok(x) = i64::try_from(x.clone()) {
        Value::from(x + i64::try_from(y)?)
    } else {
        Value::from(f64::try_from(x)? + f64::try_from(y)?)
    })
}

fn enumerate(obj: Value) -> Result<Value, minijinja::Error> {
    if let ValueKind::Seq | ValueKind::Iterable = obj.kind() {
        Ok(Value::make_object_iterable(obj, |obj| {
            Box::new(
                obj.try_iter()
                    .expect("Object must be iterable")
                    .enumerate()
                    .map(|(i, v)| Value::from(vec![Value::from(i), v])),
            )
        }))
    } else {
        Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "Cannot enumerate non-iterable value",
        ))
    }
}
