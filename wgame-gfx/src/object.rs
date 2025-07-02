use glam::Mat4;

pub struct Vertices<'a> {
    pub count: u32,
    pub vertex_buffer: &'a wgpu::Buffer,
    pub index_buffer: &'a wgpu::Buffer,
}

pub trait Object {
    fn vertices(&self) -> Vertices<'_>;
    fn create_uniforms(&self, transformation: Mat4) -> wgpu::BindGroup;
    fn pipeline(&self) -> &wgpu::RenderPipeline;
}

pub trait ObjectExt: Object + Sized {
    fn transform(self, transformation: Mat4) -> Transformed<Self> {
        Transformed {
            inner: self,
            transformation,
        }
    }
}

impl<T: Object> ObjectExt for T {}

pub struct Transformed<T> {
    pub inner: T,
    pub transformation: Mat4,
}

impl<T: Object> Object for Transformed<T> {
    fn vertices(&self) -> Vertices<'_> {
        self.inner.vertices()
    }

    fn create_uniforms(&self, transformation: Mat4) -> wgpu::BindGroup {
        self.inner
            .create_uniforms(transformation * self.transformation)
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}
