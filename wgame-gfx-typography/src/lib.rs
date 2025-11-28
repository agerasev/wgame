#![forbid(unsafe_code)]

mod library;
mod render;
mod text;
mod texture;

pub use self::{
    library::{TypographyLibrary, TypographyState},
    text::Text,
    texture::FontTexture,
};

pub use wgame_typography::{Font, FontAtlas};
