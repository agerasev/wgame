use crate::{Context, InstanceVisitor};

pub trait Object {
    type Context: Context;
    fn for_each_instance<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V);
}
