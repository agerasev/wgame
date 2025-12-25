use std::rc::Rc;

pub trait Context: 'static {
    fn bind_group(&self) -> wgpu::BindGroup;
}

pub trait Renderer<C: Context> {
    fn render(&self, ctx: &C, pass: &mut wgpu::RenderPass<'_>);
}

impl<C: Context, R: Renderer<C> + ?Sized> Renderer<C> for Rc<R> {
    fn render(&self, ctx: &C, pass: &mut wgpu::RenderPass<'_>) {
        (**self).render(ctx, pass);
    }
}
