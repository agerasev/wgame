#![forbid(unsafe_code)]

use wgpu::{MultisampleState, RenderPass};

use wgame_gfx::{Instance, State};

pub struct Text<'a> {
    state: State<'a>,
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    cache: glyphon::Cache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,
}

impl<'a> Text<'a> {
    fn new(state: &State<'a>, text: &str) -> Self {
        // Set up text renderer
        //
        let mut font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(state.device());
        let viewport = glyphon::Viewport::new(state.device(), &cache);
        let mut atlas =
            glyphon::TextAtlas::new(state.device(), state.queue(), &cache, state.format());
        let renderer = glyphon::TextRenderer::new(
            &mut atlas,
            state.device(),
            MultisampleState::default(),
            None,
        );
        let mut text_buffer =
            glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(30.0, 42.0));

        text_buffer.set_size(&mut font_system, None, None);
        text_buffer.set_text(
            &mut font_system,
            text,
            &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        Self {
            state: state.clone(),
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            renderer,
            text_buffer,
        }
    }

    fn render(&mut self, pass: &mut RenderPass) {
        self.renderer
            .prepare(
                self.state.device(),
                self.state.queue(),
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [glyphon::TextArea {
                    buffer: &self.text_buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }
}
