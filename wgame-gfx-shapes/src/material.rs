//! Custom shading on the shared mesh/instance renderer.
//!
//! A [`MeshShader`] adds typed instance attributes and bindings after the usual
//! camera (group 0) and color texture (group 1). It still uses [`PolygonFill`],
//! [`wgame_gfx::Scene`], transforms, texture atlases and depth policies. This is
//! useful for lighting, additional texture layers, or other surface effects.
//!
//! Texture bindings and [`wgame_gfx_texture::TextureAttribute`] coordinates are
//! resolved together when baking. Existing baked renderers retain their atlas
//! generation. Uniform bind groups are retained directly; updating their buffers
//! affects previously baked renderers too.
use std::{marker::PhantomData, rc::Rc};

use anyhow::{Result, ensure};
use glam::Affine2;
use wgame_gfx::{
    Camera, Instance, Object,
    prelude::*,
    types::{Color, Transform},
};
use wgame_gfx_texture::TextureResource;
use wgame_shader::Attribute;

use crate::{
    PolygonFill, ShapesState, Textured,
    pipeline::{Pipelines, create_pipeline_with_bindings},
    render::{ShapeResource, ShapeStorage},
    shader::{InstanceData, ShaderConfig, Vertex},
};

/// One extra bind group, in the order declared by [`MeshShader::new`].
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MaterialBinding {
    /// Resolved at bake time so atlas moves and resizing remain safe.
    Texture(TextureResource),
    /// A caller-owned bind group, for example a uniform buffer.
    Uniforms(wgpu::BindGroup),
}
impl MaterialBinding {
    pub(crate) fn bind_group(&self) -> wgpu::BindGroup {
        match self {
            Self::Texture(texture) => texture.bind_group(),
            Self::Uniforms(group) => group.clone(),
        }
    }
}
struct ShaderInner {
    state: ShapesState,
    pipelines: Pipelines,
    binding_count: usize,
}
/// Reusable custom mesh pipelines. `A` defines the extra instance layout.
/// Built-in vertices expose position, local coordinates, tint and surface normal.
pub struct MeshShader<A: Attribute> {
    inner: Rc<ShaderInner>,
    _attribute: PhantomData<A>,
}
impl<A: Attribute> Clone for MeshShader<A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _attribute: PhantomData,
        }
    }
}
impl<A: Attribute + Clone> MeshShader<A> {
    /// Compile a shader. `config.instance` is derived from `A`; leave it empty.
    /// Extra layouts start at group 2 and must match each material's bindings.
    /// Shader source must respect WebGL2 limits when targeting that backend.
    pub fn new(
        state: &ShapesState,
        mut config: ShaderConfig,
        layouts: &[wgpu::BindGroupLayout],
    ) -> Result<Self> {
        ensure!(
            config.instance.len() == 0,
            "instance bindings are derived from the material attribute type"
        );
        config.instance = A::bindings();
        ensure!(
            2 + layouts.len() <= state.device().limits().max_bind_groups as usize,
            "too many material bind groups"
        );
        ensure!(
            Vertex::bindings().count() + InstanceData::<A>::bindings().count()
                <= state.device().limits().max_vertex_attributes,
            "too many material vertex attributes"
        );
        Ok(Self {
            inner: Rc::new(ShaderInner {
                state: state.clone(),
                pipelines: create_pipeline_with_bindings(state, &config, layouts)?,
                binding_count: layouts.len(),
            }),
            _attribute: PhantomData,
        })
    }
    /// Bind resources and per-instance parameters to this shader.
    /// Binding layouts must match those supplied to [`Self::new`].
    pub fn material(
        &self,
        bindings: Vec<MaterialBinding>,
        attributes: A,
    ) -> Result<MeshMaterial<A>> {
        ensure!(
            bindings.len() == self.inner.binding_count,
            "material binding count differs from shader layout"
        );
        Ok(MeshMaterial {
            shader: self.clone(),
            bindings,
            attributes,
        })
    }
}
/// Shared shader/resources with per-instance attributes.
#[derive(Clone)]
pub struct MeshMaterial<A: Attribute + Clone> {
    shader: MeshShader<A>,
    bindings: Vec<MaterialBinding>,
    attributes: A,
}
/// A polygon or triangle mesh with a custom material, using ordinary scene batching.
#[derive(Clone)]
pub struct MaterialMesh<A: Attribute + Clone> {
    base: PolygonFill,
    material: MeshMaterial<A>,
}
impl PolygonFill {
    /// Replace unlit shading while keeping the mesh, color texture and depth policy.
    pub fn with_material<A: Attribute + Clone>(
        &self,
        material: &MeshMaterial<A>,
    ) -> MaterialMesh<A> {
        assert_eq!(
            self.resource().device,
            *material.shader.inner.state.device(),
            "material and geometry must use the same device"
        );
        MaterialMesh {
            base: self.clone(),
            material: material.clone(),
        }
    }
}
impl<A: Attribute + Clone> MaterialMesh<A> {
    pub fn depth(&self, depth: wgame_gfx::DepthMode) -> Self {
        Self {
            base: self.base.depth(depth),
            ..self.clone()
        }
    }
}
impl<A: Attribute + Clone> Instance for MaterialMesh<A> {
    type Context = Camera;
    type Resource = ShapeResource<A>;
    type Storage = ShapeStorage<A>;
    fn resource(&self) -> Self::Resource {
        let base = self.base.resource();
        ShapeResource {
            vertices: base.vertices,
            texture: base.texture,
            bindings: self.material.bindings.clone(),
            pipeline: self
                .material
                .shader
                .inner
                .pipelines
                .get(self.base.depth_mode()),
            device: base.device,
            _ghost: PhantomData,
        }
    }
    fn new_storage(&self) -> Self::Storage {
        ShapeStorage::new(self.resource())
    }
    fn store(&self, storage: &mut Self::Storage) {
        let base = self.base.instance_data();
        storage.instances.push(InstanceData {
            matrix: base.matrix,
            tex: base.tex,
            custom: self.material.attributes.clone(),
        });
    }
}
impl<A: Attribute + Clone> Object for MaterialMesh<A> {
    type Context = Camera;
    fn for_each_instance<V: wgame_gfx::InstanceVisitor<Camera>>(&self, visitor: &mut V) {
        visitor.visit(self);
    }
}
impl<A: Attribute + Clone> Transformable for MaterialMesh<A> {
    fn transform<X: Transform>(&self, xform: X) -> Self {
        Self {
            base: self.base.transform(xform),
            ..self.clone()
        }
    }
}
impl<A: Attribute + Clone> Colorable for MaterialMesh<A> {
    fn multiply_color<C: Color>(&self, color: C) -> Self {
        Self {
            base: self.base.multiply_color(color),
            ..self.clone()
        }
    }
}
impl<A: Attribute + Clone> Textured for MaterialMesh<A> {
    /// Transform the base color UVs. Additional textures retain their own mapping.
    fn transform_texcoord(&self, transform: Affine2) -> Self {
        Self {
            base: self.base.transform_texcoord(transform),
            ..self.clone()
        }
    }
}
