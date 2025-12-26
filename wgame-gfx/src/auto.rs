use std::ops::{Deref, DerefMut};

use crate::{Camera, Context, Frame, Scene};

pub struct AutoScene<'a, 'b, 'c, C: Context = Camera> {
    pub frame: &'c mut Frame<'a, 'b>,
    pub camera: C,
    pub items: Scene<C>,
}

impl<'a, 'b, 'c, C: Context> AutoScene<'a, 'b, 'c, C> {
    pub fn new(frame: &'c mut Frame<'a, 'b>, context: C) -> Self {
        Self {
            frame,
            camera: context,
            items: Scene::default(),
        }
    }

    pub fn discard(mut self) {
        self.items = Scene::default();
    }
}

impl<C: Context> Deref for AutoScene<'_, '_, '_, C> {
    type Target = Scene<C>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<C: Context> DerefMut for AutoScene<'_, '_, '_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<C: Context> Drop for AutoScene<'_, '_, '_, C> {
    fn drop(&mut self) {
        if !self.items.is_empty() {
            self.frame.render_iter(&self.camera, self.items.iter());
        }
    }
}
