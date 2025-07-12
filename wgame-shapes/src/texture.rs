use glam::Affine2;

use wgame_gfx::SharedState;

#[derive(Clone)]
pub struct Texture<'a> {
    pub(crate) state: SharedState<'a>,
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) xform: Affine2,
}

impl<'a> Texture<'a> {
    pub fn new(state: &SharedState<'a>, size: (u32, u32), format: wgpu::TextureFormat) -> Self {
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture = state.device().create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = state.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            state: state.clone(),
            extent,
            texture,
            view,
            sampler,
            xform: Affine2::IDENTITY,
        }
    }

    pub fn transform_coords(self, xform: Affine2) -> Self {
        Self {
            xform: xform * self.xform,
            ..self
        }
    }

    pub fn write(&self, data: &[u8]) {
        let format = self.texture.format();
        let bytes_per_block = format.block_copy_size(None).unwrap();
        assert_eq!(
            (self.extent.width * self.extent.height * bytes_per_block) as usize,
            data.len()
        );
        self.state.queue().write_texture(
            self.texture.as_image_copy(),
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(
                    (self.extent.width / format.block_dimensions().0) * bytes_per_block,
                ),
                rows_per_image: Some(self.extent.height / format.block_dimensions().1),
            },
            self.extent,
        );
    }

    pub fn with_data(
        state: &SharedState<'a>,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        data: &[u8],
    ) -> Self {
        let this = Self::new(state, size, format);
        this.write(data);
        this
    }
}
