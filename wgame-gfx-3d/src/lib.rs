//! 3D primitives built from the existing shape mesh renderer.
//!
//! No separate camera, scene, texture or rendering pipeline is needed. Construct
//! a primitive once, then clone/transform/fill it like any other shape. Sphere
//! UVs are equirectangular; cylinder caps map a disk into the full texture and
//! the side maps circumference to U and height to V. Both use a Z-up local frame.
#![forbid(unsafe_code)]
use glam::{Vec2, Vec3};
use std::f32::consts::{PI, TAU};
use wgame_gfx_shapes::{Mesh, Polygon, ShapesLibrary, shader::Vertex};

/// Solid primitives using shared meshes, fills, transforms and depth policies.
pub trait Shapes3d {
    /// Unit-radius sphere centered at the origin. At least 3 sectors and 2 rings.
    fn sphere(&self, sectors: u32, rings: u32) -> Polygon;
    /// Closed unit-radius cylinder with Z from -0.5 to 0.5. At least 3 sectors.
    fn cylinder(&self, sectors: u32) -> Polygon;
}
impl Shapes3d for ShapesLibrary {
    fn sphere(&self, sectors: u32, rings: u32) -> Polygon {
        let (vertices, indices) = sphere_mesh(sectors, rings);
        self.mesh(Mesh::from_arrays(self.state(), &vertices, Some(&indices)))
    }
    fn cylinder(&self, sectors: u32) -> Polygon {
        let (vertices, indices) = cylinder_mesh(sectors);
        self.mesh(Mesh::from_arrays(self.state(), &vertices, Some(&indices)))
    }
}
fn vertex(pos: Vec3, uv: Vec2) -> Vertex {
    Vertex::new(pos.extend(1.0), uv.extend(1.0))
}
fn sphere_mesh(sectors: u32, rings: u32) -> (Vec<Vertex>, Vec<u32>) {
    assert!(
        (3..=4096).contains(&sectors) && (2..=4096).contains(&rings),
        "invalid sphere subdivisions"
    );
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for j in 0..=rings {
        let v = j as f32 / rings as f32;
        let angle = PI * v;
        for i in 0..=sectors {
            let u = i as f32 / sectors as f32;
            let dir = Vec2::from_angle(TAU * u);
            vertices.push(vertex(
                Vec3::new(dir.x * angle.sin(), dir.y * angle.sin(), angle.cos()),
                Vec2::new(u, v),
            ));
            if j < rings && i < sectors {
                let a = j * (sectors + 1) + i;
                let b = a + sectors + 1;
                if j > 0 {
                    indices.extend([a, b, a + 1]);
                }
                if j + 1 < rings {
                    indices.extend([a + 1, b, b + 1]);
                }
            }
        }
    }
    (vertices, indices)
}
fn cylinder_mesh(sectors: u32) -> (Vec<Vertex>, Vec<u32>) {
    assert!(
        (3..=4096).contains(&sectors),
        "invalid cylinder subdivisions"
    );
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    // Separate cap/side vertices retain their distinct UVs at the rim.
    for z in [-0.5, 0.5] {
        let center = vertices.len() as u32;
        vertices.push(vertex(Vec3::new(0.0, 0.0, z), Vec2::splat(0.5)));
        for i in 0..=sectors {
            let p = Vec2::from_angle(TAU * i as f32 / sectors as f32);
            vertices.push(vertex(p.extend(z), (p + Vec2::ONE) * 0.5));
            if i < sectors {
                let (a, b) = (center + i + 1, center + i + 2);
                indices.extend(if z > 0.0 {
                    [center, a, b]
                } else {
                    [center, b, a]
                });
            }
        }
    }
    let offset = vertices.len() as u32;
    for i in 0..=sectors {
        let u = i as f32 / sectors as f32;
        let p = Vec2::from_angle(TAU * u);
        for (z, v) in [(-0.5, 1.0), (0.5, 0.0)] {
            vertices.push(vertex(p.extend(z), Vec2::new(u, v)));
        }
        if i < sectors {
            let a = offset + 2 * i;
            indices.extend([a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
    }
    (vertices, indices)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn primitives_have_valid_uvs_bounds_and_outward_triangles() {
        for (vertices, indices) in [sphere_mesh(16, 8), cylinder_mesh(16)] {
            for v in &vertices {
                assert!(v.pos.is_finite());
                assert!((0.0..=1.0).contains(&v.local_coord.x));
                assert!((0.0..=1.0).contains(&v.local_coord.y));
                assert!(v.pos.truncate().abs().max_element() <= 1.00001);
            }
            for tri in indices.as_chunks::<3>().0 {
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| vertices[i as usize].pos.truncate());
                let normal = (b - a).cross(c - a);
                assert!(normal.length() > 1e-6);
                assert!(normal.dot(a + b + c) > 0.0);
            }
        }
    }
}
