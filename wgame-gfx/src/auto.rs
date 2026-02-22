use std::ops::{Deref, DerefMut};

use crate::{Camera, Context, Scene, Target};

pub struct AutoScene<'a, T: Target + ?Sized, C: Context = Camera> {
    pub target: &'a mut T,
    pub camera: C,
    pub items: Scene<C>,
}

impl<'a, T: Target + ?Sized, C: Context> AutoScene<'a, T, C> {
    pub fn new(target: &'a mut T, context: C) -> Self {
        Self {
            target,
            camera: context,
            items: Scene::default(),
        }
    }

    pub fn discard(mut self) {
        self.items = Scene::default();
    }
}

impl<T: Target + ?Sized, C: Context> Deref for AutoScene<'_, T, C> {
    type Target = Scene<C>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T: Target + ?Sized, C: Context> DerefMut for AutoScene<'_, T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T: Target + ?Sized, C: Context> Drop for AutoScene<'_, T, C> {
    fn drop(&mut self) {
        if !self.items.is_empty() {
            self.target.render_iter(&self.camera, self.items.iter());
        }
    }
}
