pub trait Context {
    fn bind_group(&self) -> wgpu::BindGroup;
}

pub trait Renderer<C: Context> {
    fn render(&self, ctx: &C, pass: &mut wgpu::RenderPass<'_>);
}
