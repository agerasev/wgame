use alloc::{boxed::Box, string::String, vec, vec::Vec};

use anyhow::Result;
use minijinja::{Environment, UndefinedBehavior, Value, value::ValueKind};
use serde::Serialize;

use crate::{attributes::AttributeList, binding::BindingInfo};

#[derive(Clone, Default, Debug, Serialize)]
pub struct ShaderConfig {
    pub fragment_modifier: String,
    pub instances: AttributeList,
    pub uniforms: Vec<BindingInfo>,
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
        log::debug!(
            "Shader '{}' substitution:\nConfig: {:?}\nSource:\n{}",
            self.name,
            ctx,
            rendered
        );
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
