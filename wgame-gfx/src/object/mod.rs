pub struct Vertices {
    pub count: u32,
    pub buffer: wgpu::Buffer,
}

pub struct Uniforms {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub trait Object {
    fn vertices(&self) -> Vertices;
    fn uniforms(&self) -> Uniforms;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}
