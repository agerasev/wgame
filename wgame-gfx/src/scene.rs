use crate::{
    AnyResource, AnyStorage, Camera, Context, Instance, InstanceVisitor, Object, Resource,
};
use smallvec::SmallVec;
use std::{any::Any, cmp::Ordering, collections::BTreeMap, rc::Rc};

// Missing trailing components have order zero, including negative nested orders.
#[derive(Clone)]
struct OrderKey(SmallVec<[i32; 4]>);
impl PartialEq for OrderKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for OrderKey {}
impl Ord for OrderKey {
    fn cmp(&self, other: &Self) -> Ordering {
        (0..self.0.len().max(other.0.len()))
            .map(|i| {
                self.0
                    .get(i)
                    .copied()
                    .unwrap_or(0)
                    .cmp(&other.0.get(i).copied().unwrap_or(0))
            })
            .find(|ord| *ord != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for OrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
struct Batch<C: Context> {
    resource: Rc<dyn AnyResource>,
    storage: Box<dyn AnyStorage<C>>,
}

/// Painter-ordered scene. Lower explicit orders render first; equal orders preserve
/// insertion order. Only adjacent compatible instances within an order are batched.
pub struct Scene<C: Context = Camera> {
    layers: BTreeMap<OrderKey, Vec<Batch<C>>>,
    batches: usize,
}
impl<C: Context> Default for Scene<C> {
    fn default() -> Self {
        Self {
            layers: BTreeMap::new(),
            batches: 0,
        }
    }
}
impl<C: Context> Scene<C> {
    fn add_instance<T: Instance<Context = C>>(&mut self, instance: &T) {
        let resource = instance.resource();
        let mut order: SmallVec<[i32; 4]> = resource.order().collect();
        while order.last() == Some(&0) {
            order.pop();
        }
        let layer = self.layers.entry(OrderKey(order)).or_default();
        if let Some(last) = layer.last_mut()
            && last.resource.as_ref().eq_dyn(&resource)
            && let Some(storage) =
                (last.storage.as_mut() as &mut dyn Any).downcast_mut::<T::Storage>()
        {
            instance.store(storage);
            return;
        }
        let mut storage = instance.new_storage();
        instance.store(&mut storage);
        layer.push(Batch {
            resource: resource.clone_dyn(),
            storage: Box::new(storage),
        });
        self.batches += 1;
    }
    /// Bake fixed instance buffers and resource bindings for repeated rendering.
    /// Built-in shape/text storage resolves current atlas coordinates here. Existing
    /// baked drawing survives atlas relocation and source texture drop/resize;
    /// rebake this scene to follow current allocations. Rebuild the source scene
    /// when object data changes. Explicit texture pixel updates are not frozen.
    pub fn bake(&self) -> BakedScene<C> {
        BakedScene {
            renderers: self.iter().map(|storage| storage.bake_dyn()).collect(),
        }
    }
    pub fn add<T: Object<Context = C>>(&mut self, object: &T) {
        object.for_each_instance(self);
    }
    pub fn is_empty(&self) -> bool {
        self.batches == 0
    }
    /// Number of ordered draw batches, not objects or render passes.
    pub fn len(&self) -> usize {
        self.batches
    }
    pub fn iter(&self) -> impl Iterator<Item = &dyn AnyStorage<C>> {
        self.layers
            .values()
            .flat_map(|layer| layer.iter().map(|batch| batch.storage.as_ref()))
    }
}
impl<C: Context> InstanceVisitor<C> for Scene<C> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T) {
        self.add_instance(instance);
    }
}

/// Renderer snapshot with fixed instance buffers and resource bindings.
/// Built-in renderers retain their GPU atlas generation across relocation;
/// explicit updates to pixels in that generation remain mutable.
pub struct BakedScene<C: Context = Camera> {
    renderers: Vec<Rc<dyn crate::Renderer<C>>>,
}
impl<C: Context> BakedScene<C> {
    pub fn len(&self) -> usize {
        self.renderers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.renderers.is_empty()
    }
}
impl<C: Context> crate::Renderer<C> for BakedScene<C> {
    fn render(&self, ctx: &C, pass: &mut wgpu::RenderPass<'_>) {
        for renderer in &self.renderers {
            renderer.render(ctx, pass);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Renderer, Storage};
    struct Ctx;
    impl Context for Ctx {
        fn bind_group(&self) -> wgpu::BindGroup {
            unreachable!()
        }
    }
    #[derive(Clone, Eq, PartialEq, Hash)]
    struct Res(u8);
    impl Resource for Res {}
    struct Item(u8, u8);
    struct Stored {
        resource: Res,
        ids: Vec<u8>,
    }
    struct Draw;
    impl Renderer<Ctx> for Draw {
        fn render(&self, _: &Ctx, _: &mut wgpu::RenderPass<'_>) {}
    }
    impl Storage for Stored {
        type Context = Ctx;
        type Resource = Res;
        type Renderer = Draw;
        fn resource(&self) -> Res {
            self.resource.clone()
        }
        fn bake(&self) -> Draw {
            Draw
        }
    }
    impl Instance for Item {
        type Context = Ctx;
        type Resource = Res;
        type Storage = Stored;
        fn resource(&self) -> Res {
            Res(self.0)
        }
        fn new_storage(&self) -> Stored {
            Stored {
                resource: self.resource(),
                ids: Vec::new(),
            }
        }
        fn store(&self, dst: &mut Stored) {
            dst.ids.push(self.1);
        }
    }
    #[test]
    fn alternating_resources_preserve_painter_order() {
        let mut scene = Scene::default();
        for item in [Item(1, 1), Item(2, 2), Item(1, 3), Item(1, 4)] {
            scene.visit(&item);
        }
        let ids: Vec<Vec<u8>> = scene
            .iter()
            .map(|s| {
                (s as &dyn Any)
                    .downcast_ref::<Stored>()
                    .unwrap()
                    .ids
                    .clone()
            })
            .collect();
        assert_eq!(ids, [vec![1], vec![2], vec![3, 4]]);
        assert_eq!(scene.len(), 3);
    }
    #[test]
    fn nested_order_is_zero_padded_and_stable() {
        let mut scene = Scene::default();
        scene.visit(&crate::Ordered::new(Item(1, 1), 1));
        scene.visit(&Item(2, 2));
        scene.visit(&crate::Ordered::new(crate::Ordered::new(Item(3, 3), -1), 0));
        scene.visit(&crate::Ordered::new(Item(4, 4), -1));
        let orders: Vec<_> = scene.iter().map(|s| s.resource_dyn().order_dyn()).collect();
        assert_eq!(
            orders.iter().map(|x| x.as_slice()).collect::<Vec<_>>(),
            [&[-1][..], &[0, -1], &[], &[1]]
        );
        assert_eq!(
            OrderKey(SmallVec::new()).cmp(&OrderKey(smallvec::smallvec![0])),
            Ordering::Equal
        );
    }
}
