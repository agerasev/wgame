use std::{
    any::Any,
    cmp::Ordering,
    hash::{Hash, Hasher},
    rc::Rc,
};

use anyhow::Result;

use crate::{renderer::Renderer, utils::AnyKey};

/// Shared resource required to draw an instance.
///
/// Equality of the instances' resource means that they can be draw in single render pass.
pub trait Resource: Any + Ord + Hash + Clone + Sized {
    type Storage: Any;
    type Renderer: Renderer + Ord + Hash;

    fn new_storage(&self) -> Self::Storage;
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer>;

    fn as_any(&self) -> &dyn AnyResource {
        self
    }
    fn into_any(self) -> Rc<dyn AnyResource> {
        Rc::new(self)
    }
}

pub trait AnyResource: AnyKey {
    fn clone_dyn(&self) -> Rc<dyn AnyResource>;
    fn new_dyn_storage(&self) -> Box<dyn Any>;
    fn make_dyn_renderer(&self, instances: &dyn Any) -> Result<Box<dyn Renderer>>;
}

impl<R: Resource> AnyResource for R {
    fn clone_dyn(&self) -> Rc<dyn AnyResource> {
        Rc::new(self.clone())
    }

    fn new_dyn_storage(&self) -> Box<dyn Any> {
        Box::new(self.new_storage())
    }

    fn make_dyn_renderer(&self, instances: &dyn Any) -> Result<Box<dyn Renderer>> {
        let instances = instances
            .downcast_ref::<R::Storage>()
            .expect("Error downcasting storage during draw");
        Ok(Box::new(self.make_renderer(instances)?))
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
    type Renderer = Box<dyn Renderer>;

    fn new_storage(&self) -> Self::Storage {
        (**self).new_dyn_storage()
    }
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        (**self).make_dyn_renderer(&**instances)
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
}

impl From<&dyn AnyResource> for Rc<dyn AnyResource> {
    fn from(value: &dyn AnyResource) -> Self {
        value.clone_dyn()
    }
}
