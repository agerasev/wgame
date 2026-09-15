use glam::Vec3;
use wgame_gfx_shapes::shader::Vertex;

/// Invalid triangle mesh supplied to [`recalculate_normals`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NormalError {
    #[error("triangle index count must be a multiple of three")]
    IncompleteTriangle,
    #[error("triangle index is outside the vertex array")]
    InvalidIndex,
    #[error("mesh positions must be finite affine points (w = 1)")]
    InvalidPosition,
}
/// Area-weighted normals, smoothing only vertices that share an index.
///
/// Keep UV seams and hard edges split; preserve imported normals when available.
/// Unused vertices and zero-area triangles fall back to +Z. All input validation
/// happens before mutation. Non-indexed meshes can supply `0..vertices.len()`.
pub fn recalculate_normals(vertices: &mut [Vertex], indices: &[u32]) -> Result<(), NormalError> {
    if !indices.len().is_multiple_of(3) {
        return Err(NormalError::IncompleteTriangle);
    }
    if indices.iter().any(|&i| i as usize >= vertices.len()) {
        return Err(NormalError::InvalidIndex);
    }
    if vertices
        .iter()
        .any(|v| !v.pos.is_finite() || v.pos.w != 1.0)
    {
        return Err(NormalError::InvalidPosition);
    }
    let mut normals = vec![Vec3::ZERO; vertices.len()];
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        let [p, q, r] = [a, b, c].map(|i| vertices[i as usize].pos.truncate());
        let n = (q - p).cross(r - p);
        for i in [a, b, c] {
            normals[i as usize] += n;
        }
    }
    for (vertex, normal) in vertices.iter_mut().zip(normals) {
        vertex.normal = normal.try_normalize().unwrap_or(Vec3::Z);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normals_follow_winding_and_reject_invalid_meshes_before_mutation() {
        let mut vertices: Vec<_> = [Vec3::ZERO, Vec3::Y, Vec3::Z]
            .map(|p| Vertex::new(p.extend(1.0), Vec3::Z))
            .into();
        recalculate_normals(&mut vertices, &[0, 1, 2]).unwrap();
        assert!(vertices.iter().all(|v| v.normal == Vec3::X));
        assert_eq!(
            recalculate_normals(&mut vertices, &[0, 1, 3]),
            Err(NormalError::InvalidIndex)
        );
        assert!(vertices.iter().all(|v| v.normal == Vec3::X));
        recalculate_normals(&mut vertices, &[0, 2, 1]).unwrap();
        assert!(vertices.iter().all(|v| v.normal == -Vec3::X));
        recalculate_normals(&mut vertices, &[0, 0, 0]).unwrap();
        assert!(vertices.iter().all(|v| v.normal == Vec3::Z));
    }
}
