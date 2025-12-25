use std::{any::Any, rc::Rc};

use crate::{AnyResource, Context, Renderer, Resource};

/// Single instance to draw.
pub trait Instance {
    type Context: Context;
    type Resource: Resource;
    type Storage: Storage<Context = Self::Context, Resource = Self::Resource>;

    fn resource(&self) -> Self::Resource;
    fn new_storage(&self) -> Self::Storage;
    fn store(&self, storage: &mut Self::Storage);
}

pub trait Storage: Any {
    type Context: Context;
    type Resource: Resource;
    type Renderer: Renderer<Self::Context>;

    fn resource(&self) -> Self::Resource;
    /// Bake all collected instances into a single immutable renderer.
    fn bake(&self) -> Self::Renderer;
}

pub trait AnyStorage<C: Context>: Any + 'static {
    fn resource_dyn(&self) -> Rc<dyn AnyResource>;
    fn bake_dyn(&self) -> Rc<dyn Renderer<C>>;
}

impl<S: Storage> AnyStorage<S::Context> for S {
    fn resource_dyn(&self) -> Rc<dyn AnyResource> {
        self.resource().clone_dyn()
    }
    fn bake_dyn(&self) -> Rc<dyn Renderer<S::Context>> {
        Rc::new(self.bake())
    }
}

impl<C: Context> Storage for dyn AnyStorage<C> {
    type Context = C;
    type Resource = Rc<dyn AnyResource>;
    type Renderer = Rc<dyn Renderer<C>>;

    fn resource(&self) -> Self::Resource {
        self.resource_dyn()
    }
    fn bake(&self) -> Self::Renderer {
        self.bake_dyn()
    }
}

impl<S: Storage + ?Sized> Renderer<S::Context> for S {
    fn render(&self, ctx: &S::Context, pass: &mut wgpu::RenderPass<'_>) {
        self.bake().render(ctx, pass);
    }
}
