use glam::{Vec3, Vec4};
use wgame_gfx::{Graphics, types::Color};
use wgame_gfx_shapes::{
    ShapesLibrary,
    material::{MaterialBinding, MaterialMesh, MeshMaterial, MeshShader},
    shader::ShaderConfig,
};
use wgame_gfx_texture::{Texture, TextureAttribute, TexturingLibrary};
use wgame_image::Image;
use wgame_shader::Attribute;
use wgpu::util::DeviceExt;

/// Invalid light or surface parameters. Values must be finite; intensities and
/// normal strength nonnegative, and shininess at least one.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("invalid lighting parameter: {0}")]
pub struct LightingError(pub &'static str);

/// World-space light and viewer. RGB intensities use linear light units.
#[derive(Clone, Copy, Debug)]
pub struct LightParameters {
    pub ambient: Vec3,
    /// Direction from a surface toward the light. Must be nonzero.
    pub direction: Vec3,
    pub color: Vec3,
    /// Eye position for perspective specular highlights.
    pub eye: Vec3,
}
impl Default for LightParameters {
    fn default() -> Self {
        Self {
            ambient: Vec3::splat(0.2),
            direction: Vec3::new(-0.5, -1.0, 1.0),
            color: Vec3::splat(0.8),
            eye: Vec3::new(0.0, 0.0, 10.0),
        }
    }
}
impl LightParameters {
    fn packed(self) -> Result<[Vec4; 4], LightingError> {
        for (name, rgb) in [("ambient", self.ambient), ("color", self.color)] {
            if !rgb.is_finite() || rgb.min_element() < 0.0 {
                return Err(LightingError(name));
            }
        }
        let direction = self
            .direction
            .try_normalize()
            .ok_or(LightingError("direction"))?;
        if !self.eye.is_finite() {
            return Err(LightingError("eye"));
        }
        Ok([
            self.ambient.extend(0.0),
            direction.extend(0.0),
            self.color.extend(0.0),
            self.eye.extend(0.0),
        ])
    }
}
/// Green-channel convention in a tangent-space normal map.
#[derive(Clone, Copy, Debug, Default)]
pub enum NormalY {
    /// Green above 0.5 points along increasing texture V.
    #[default]
    Positive,
    /// Flip the green channel for maps authored with the opposite convention.
    Negative,
}
/// Blinn–Phong surface parameters. Vertex/instance tints are linear multipliers.
#[derive(Clone, Copy, Debug)]
pub struct MaterialSettings {
    pub specular: f32,
    pub shininess: f32,
    pub normal_strength: f32,
    pub normal_y: NormalY,
    /// Decode the base color texture from sRGB. Normal maps are always raw data.
    pub albedo_srgb: bool,
}
impl Default for MaterialSettings {
    fn default() -> Self {
        Self {
            specular: 0.15,
            shininess: 32.0,
            normal_strength: 1.0,
            normal_y: NormalY::Positive,
            albedo_srgb: true,
        }
    }
}
impl MaterialSettings {
    fn validate(self) -> Result<(), LightingError> {
        for (name, value, minimum) in [
            ("specular", self.specular, 0.0),
            ("shininess", self.shininess, 1.0),
            ("normal_strength", self.normal_strength, 0.0),
        ] {
            if !value.is_finite() || value < minimum {
                return Err(LightingError(name));
            }
        }
        Ok(())
    }
}
#[doc(hidden)]
#[derive(Clone, Attribute)]
pub struct LightingAttributes {
    normal_tex: TextureAttribute,
    settings: Vec4,
    surface: glam::Vec2,
}
#[derive(Attribute)]
struct Varyings {
    world_position: Vec3,
    world_normal: Vec3,
    normal_coord: glam::Vec2,
    settings: Vec4,
    surface: glam::Vec2,
}
pub type LitMaterial = MeshMaterial<LightingAttributes>;
pub type LitMesh = MaterialMesh<LightingAttributes>;

/// Reusable lighting pipelines, shared normal-map fallback and light uniforms.
///
/// Uses the same meshes, cameras and scenes as unlit drawing. Normals transform
/// by the inverse transpose, including nonuniform/reflected scales. Tangent
/// directions come from position/UV derivatives; mirrored UVs work without a
/// tangent vertex stream. Degenerate UVs fall back to the mesh normal.
///
/// Normal maps encode XYZ as RGB in `0..1`, with `(0.5,0.5,1)` neutral. Upload
/// their raw decoded pixels without sRGB conversion. Their UV transform is
/// independent of the base color texture. Derivative tangents need UV seams and
/// hard edges split; they are not a MikkTSpace tangent import path.
///
/// Mesh positions must be affine points (`w = 1`).
/// Lighting is computed in linear space. sRGB targets encode the output; other
/// targets receive explicitly sRGB-encoded RGB. Blending retains the renderer's
/// usual target-space behavior. Transparent surfaces still require depth/read
/// policy and back-to-front scene ordering.
///
/// ```no_run
/// # fn draw(shapes: &wgame_gfx_shapes::ShapesLibrary, textures: &wgame_gfx_texture::TexturingLibrary,
/// # base: &wgame_gfx_texture::Texture, normal: &wgame_gfx_texture::Texture) -> anyhow::Result<()> {
/// use wgame_gfx_3d::{Lighting, LightParameters, MaterialSettings, Shapes3d};
/// use wgame_gfx_shapes::prelude::*;
/// let lighting = Lighting::new(shapes, textures, LightParameters::default())?;
/// let material = lighting.material(Some(normal), MaterialSettings::default())?;
/// let object = shapes.sphere(32,16).fill_texture(base).with_material(&material);
/// let mut scene = wgame_gfx::Scene::default();
/// scene.add(&object);
/// # Ok(()) }
/// ```
pub struct Lighting {
    state: Graphics,
    buffer: wgpu::Buffer,
    uniforms: wgpu::BindGroup,
    shader: MeshShader<LightingAttributes>,
    neutral: Texture,
}
impl Lighting {
    pub fn new(
        shapes: &ShapesLibrary,
        textures: &TexturingLibrary,
        params: LightParameters,
    ) -> anyhow::Result<Self> {
        let data = params.packed()?;
        let state: &Graphics = shapes.state();
        anyhow::ensure!(
            state == &**textures.state(),
            "lighting and textures must share a graphics context"
        );
        let layout = state
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("lighting"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZero::new(64),
                    },
                    count: None,
                }],
            });
        let buffer = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lighting"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let uniforms = state
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lighting"),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
        let neutral = textures.texture(
            &Image::with_color((1, 1), Vec4::new(0.5, 0.5, 1.0, 1.0).to_rgba_f16()),
            Default::default(),
        );
        let shader = MeshShader::new(
            shapes.state(),
            ShaderConfig {
                varying: Varyings::bindings(),
                module_source: include_str!("lighting.wgsl").into(),
                vertex_source: include_str!("lighting_vertex.wgsl").into(),
                fragment_color_source: format!(
                    "{}\ncolor = vec4<f32>({}, color.a);",
                    include_str!("lighting_fragment.wgsl"),
                    if state.format().is_srgb() {
                        "lit_color"
                    } else {
                        "linear_to_srgb(lit_color)"
                    }
                ),
                ..Default::default()
            },
            &[
                shapes.state().texture().float_bind_group_layout.clone(),
                layout,
            ],
        )?;
        Ok(Self {
            state: state.clone(),
            buffer,
            uniforms,
            shader,
            neutral,
        })
    }
    /// Update before encoding/submitting a frame. Baked scenes observe these
    /// uniforms too. Use separate Lighting objects for independently lit views
    /// in one submission; multiple queued writes are not per-draw snapshots.
    pub fn update(&self, params: LightParameters) -> Result<(), LightingError> {
        self.state
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&params.packed()?));
        Ok(())
    }
    /// Material with an optional normal map. A missing map uses a neutral normal.
    /// The normal texture's color multiplier is ignored; only its UVs/data matter.
    pub fn material(
        &self,
        normal: Option<&Texture>,
        settings: MaterialSettings,
    ) -> anyhow::Result<LitMaterial> {
        settings.validate()?;
        let normal = normal.unwrap_or(&self.neutral);
        self.shader.material(
            vec![
                MaterialBinding::Texture(normal.resource()),
                MaterialBinding::Uniforms(self.uniforms.clone()),
            ],
            LightingAttributes {
                normal_tex: normal.attribute(),
                settings: Vec4::new(
                    settings.normal_strength,
                    if matches!(settings.normal_y, NormalY::Positive) {
                        1.0
                    } else {
                        -1.0
                    },
                    f32::from(settings.albedo_srgb),
                    0.0,
                ),
                surface: glam::Vec2::new(settings.specular, settings.shininess),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_parameters_are_rejected_and_webgl_attribute_budget_is_respected() {
        assert_eq!(
            LightParameters {
                direction: Vec3::ZERO,
                ..Default::default()
            }
            .packed(),
            Err(LightingError("direction"))
        );
        assert_eq!(
            LightParameters {
                ambient: -Vec3::ONE,
                ..Default::default()
            }
            .packed(),
            Err(LightingError("ambient"))
        );
        assert_eq!(
            MaterialSettings {
                normal_strength: f32::NAN,
                ..Default::default()
            }
            .validate(),
            Err(LightingError("normal_strength"))
        );
        let count = wgame_gfx_shapes::shader::Vertex::bindings().count()
            + wgame_gfx_shapes::shader::InstanceData::<LightingAttributes>::bindings().count();
        assert!(count <= wgpu::Limits::downlevel_webgl2_defaults().max_vertex_attributes);
    }
}
