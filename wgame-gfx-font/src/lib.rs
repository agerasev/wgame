#![forbid(unsafe_code)]

mod library;
mod render;
mod text;
mod texture;

pub use self::{
    library::{TextLibrary, TextState},
    text::Text,
    texture::FontTexture,
};

pub use wgame_font::{Font, FontAtlas};
