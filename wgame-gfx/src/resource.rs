use std::{
    any::Any,
    cmp::Ordering,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::utils::AnyKey;

/// Shared resource required to draw an instance.
///
/// Equality of the instances' resource means that they can be draw in single render pass.
pub trait Resource: Any + Ord + Hash + Clone + Sized {
    type Storage: Any;

    fn new_storage(&self) -> Self::Storage;
    fn render(&self, storage: &Self::Storage, pass: &mut wgpu::RenderPass<'_>);

    fn as_any(&self) -> &dyn AnyResource {
        self
    }
    fn into_any(self) -> Rc<dyn AnyResource> {
        Rc::new(self)
    }

    fn order(&self) -> i64 {
        0
    }
}

pub trait AnyResource: AnyKey {
    fn clone_dyn(&self) -> Rc<dyn AnyResource>;
    fn new_dyn_storage(&self) -> Box<dyn Any>;
    fn render_dyn(&self, storage: &dyn Any, pass: &mut wgpu::RenderPass<'_>);
    fn order_dyn(&self) -> i64;
}

impl<R: Resource> AnyResource for R {
    fn clone_dyn(&self) -> Rc<dyn AnyResource> {
        Rc::new(self.clone())
    }

    fn new_dyn_storage(&self) -> Box<dyn Any> {
        Box::new(self.new_storage())
    }

    fn render_dyn(&self, storage: &dyn Any, pass: &mut wgpu::RenderPass<'_>) {
        self.render(
            storage
                .downcast_ref::<R::Storage>()
                .expect("Error downcasting storage"),
            pass,
        );
    }

    fn order_dyn(&self) -> i64 {
        self.order()
    }
}

impl PartialEq for dyn AnyResource {
    fn eq(&self, other: &dyn AnyResource) -> bool {
        self.eq_dyn(other)
    }
}
impl Eq for dyn AnyResource {}
impl PartialOrd for dyn AnyResource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn AnyResource {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(other)
    }
}
impl Hash for dyn AnyResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

impl Resource for Rc<dyn AnyResource> {
    type Storage = Box<dyn Any>;

    fn new_storage(&self) -> Self::Storage {
        (**self).new_dyn_storage()
    }
    fn render(&self, storage: &Self::Storage, pass: &mut wgpu::RenderPass<'_>) {
        (**self).render_dyn(storage, pass);
    }

    fn as_any(&self) -> &dyn AnyResource {
        &**self
    }
    fn into_any(self) -> Rc<dyn AnyResource>
    where
        Self: Sized,
    {
        self
    }

    fn order(&self) -> i64 {
        (**self).order_dyn()
    }
}

impl From<&dyn AnyResource> for Rc<dyn AnyResource> {
    fn from(value: &dyn AnyResource) -> Self {
        value.clone_dyn()
    }
}
