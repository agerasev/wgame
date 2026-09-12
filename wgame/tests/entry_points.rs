//! Compile the public entry points in a consumer crate; never open a window.
#![allow(dead_code)]
mod application {
    #[wgame::app]
    async fn main() {}
}
mod window {
    #[wgame::window(size = (800, 600))]
    async fn main(_window: wgame::Window<'_>) -> wgame::Result<()> {
        Ok(())
    }
}
#[derive(wgame::shader::Attribute)]
struct Tuple(f32, wgame::glam::Vec2);
#[derive(wgame::shader::Attribute)]
struct Named {
    position: wgame::glam::Vec2,
    opacity: f32,
}
#[test]
fn derives_serialize_in_field_order() {
    use wgame::shader::Attribute;
    let tuple = Tuple(1.0, wgame::glam::Vec2::new(2.0, 3.0));
    let named = Named {
        position: wgame::glam::Vec2::new(1.0, 2.0),
        opacity: 3.0,
    };
    assert_eq!(tuple.to_bytes(), named.to_bytes());
    assert_eq!(Tuple::bindings().size() as usize, tuple.to_bytes().len());
}
