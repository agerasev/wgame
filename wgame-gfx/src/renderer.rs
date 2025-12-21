pub trait Context {}

pub trait Renderer<C: Context> {
    fn render(&self, ctx: &C, pass: &mut wgpu::RenderPass<'_>);
}
