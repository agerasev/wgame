use anyhow::Result;
use minijinja::{Environment, UndefinedBehavior};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SubstContext {
    pub mask_expr: String,
}

impl Default for SubstContext {
    fn default() -> Self {
        Self {
            mask_expr: "true".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShaderSource {
    env: Environment<'static>,
}

impl ShaderSource {
    const TEMPLATE_NAME: &str = "shader_source";

    pub fn new(source: impl Into<String>) -> Result<Self> {
        let mut env = Environment::new();

        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_trim_blocks(true);
        env.set_lstrip_blocks(true);
        env.set_keep_trailing_newline(true);

        env.add_template_owned(Self::TEMPLATE_NAME, source.into())?;

        Ok(Self { env })
    }

    pub fn substitute(&self, ctx: &SubstContext) -> Result<String> {
        let template = self.env.get_template(Self::TEMPLATE_NAME)?;
        let rendered = template.render(ctx)?;
        Ok(rendered)
    }
}
