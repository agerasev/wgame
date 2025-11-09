#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod state;
mod texel;
mod texture;

use alloc::vec::Vec;
use half::f16;
use rgb::Rgba;
use wgame_gfx::{Graphics, types::Color};
use wgame_image::{Image, ImageBase, ImageWriteMut};

pub use self::{
    state::TextureState,
    texture::{Texture, TextureAtlas, TextureResources},
};

#[derive(Clone)]
pub struct TextureLibrary {
    state: TextureState,
    default_atlas: TextureAtlas,
}

impl TextureLibrary {
    pub fn new(state: &Graphics) -> Self {
        let state = TextureState::new(state);
        Self {
            default_atlas: TextureAtlas::new(
                &state,
                Default::default(),
                wgpu::TextureFormat::Rgba16Float,
            ),
            state,
        }
    }

    pub fn state(&self) -> &TextureState {
        &self.state
    }

    pub fn texture(&self, image: &Image<Rgba<f16>>) -> Texture {
        let texture = self.default_atlas.allocate(image.size());
        texture.update(|mut dst| dst.copy_from(image));
        texture
    }

    pub fn gradient<T: Color, const N: usize>(&self, colors: [T; N]) -> Texture {
        self.texture(&Image::with_data(
            (N as u32, 1),
            colors.map(|c| c.to_rgba()),
        ))
    }

    pub fn gradient2<T: Color, const M: usize, const N: usize>(
        &self,
        colors: [[T; M]; N],
    ) -> Texture {
        self.texture(&Image::with_data(
            (M as u32, N as u32),
            colors
                .into_iter()
                .flatten()
                .map(|c| c.to_rgba())
                .collect::<Vec<_>>(),
        ))
    }
}
