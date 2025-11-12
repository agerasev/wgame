use crate::{
    Collector, Context, Instance,
    modifiers::{Colored, Transformed},
    types::{Color, Transform},
};

pub trait Object {
    fn collect_into(&self, ctx: &Context, collector: &mut Collector);
}

impl<T: Object> Object for &'_ T {
    fn collect_into(&self, ctx: &Context, collector: &mut Collector) {
        (*self).collect_into(ctx, collector);
    }
}

impl<T: Instance> Object for Option<T> {
    fn collect_into(&self, ctx: &Context, collector: &mut Collector) {
        if let Some(instance) = self {
            collector.push(ctx, instance)
        }
    }
}

impl Object for () {
    fn collect_into(&self, _: &Context, _: &mut Collector) {}
}

pub trait ObjectExt: Object + Sized {
    fn transform<T: Transform>(&self, xform: T) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
    fn color<C: Color>(&self, color: C) -> Colored<&Self> {
        Colored::new(self, color)
    }
}

impl<T: Object> ObjectExt for T {}
