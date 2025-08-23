use alloc::boxed::Box;
use anyhow::Result;

pub trait Renderer {
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()>;
}

impl Renderer for Box<dyn Renderer> {
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        (**self).draw(pass)
    }
}
