use super::*;
use glam::Vec3;

fn point(x: f32, y: f32, width: f32) -> PolylinePoint {
    PolylinePoint {
        position: Vec2::new(x, y),
        width,
    }
}

fn corners(piece: &Piece) -> [Vec2; 4] {
    [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z].map(|p| piece.xform.transform_point3(p).truncate())
}

fn uv(piece: &Piece) -> [Vec2; 4] {
    [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y].map(|p| piece.texcoord.transform_point2(p))
}

fn close(actual: Vec2, expected: Vec2) {
    assert!(
        (actual - expected).length() < 1e-4,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn miter_shares_positions_and_indexed_uvs_with_a_tapered_tip() {
    let pieces = tessellate(&[
        point(-10.0, 0.0, 4.0),
        point(0.0, 0.0, 6.0),
        point(0.0, 20.0, 0.0),
    ]);
    assert_eq!(pieces.len(), 2);
    let a = corners(&pieces[0]);
    let b = corners(&pieces[1]);
    close(a[0], Vec2::new(-10.0, 2.0));
    close(a[3], Vec2::new(-10.0, -2.0));
    close(a[1], Vec2::new(-3.0, 3.0));
    close(a[2], Vec2::new(3.0, -3.0));
    close(a[1], b[0]);
    close(a[2], b[3]);
    close(b[1], Vec2::new(0.0, 20.0));
    close(b[2], b[1]);
    assert_eq!(uv(&pieces[0])[1], Vec2::new(0.5, 0.0));
    assert_eq!(uv(&pieces[0])[2], Vec2::new(0.5, 1.0));
    assert_eq!(uv(&pieces[0])[1], uv(&pieces[1])[0]);
    assert_eq!(uv(&pieces[0])[2], uv(&pieces[1])[3]);
    assert_eq!(uv(&pieces[1])[2], Vec2::ONE);
}

#[test]
fn bevel_has_shared_edges_and_constant_junction_u_for_both_turns() {
    for sign in [-1.0, 1.0] {
        let pieces = tessellate(&[
            point(-100.0, 0.0, 4.0),
            point(0.0, 0.0, 4.0),
            point(-100.0, sign * 20.0, 4.0),
        ]);
        assert_eq!(pieces.len(), 3);
        let before = corners(&pieces[0]);
        let bevel = corners(&pieces[1]);
        let after = corners(&pieces[2]);
        let outer = if sign > 0.0 { 1 } else { 0 };
        let inner = 1 - outer;
        let before_edge = [before[1], before[2]];
        let after_edge = [after[0], after[3]];
        close(bevel[0], before_edge[outer]);
        close(bevel[1], before_edge[inner]);
        close(bevel[2], after_edge[inner]);
        close(bevel[3], after_edge[outer]);
        let mapped = uv(&pieces[1]);
        assert!(mapped.iter().all(|p| p.x == 0.5));
        assert_eq!(mapped[0].y, outer as f32);
        assert_eq!(mapped[1].y, inner as f32);
        close(bevel[0], Vec2::new(0.0, -sign * 2.0));
        assert!(bevel[3].length() < 2.001);
    }
}

#[test]
fn miter_limit_is_four_half_widths_not_a_right_angle_cutoff() {
    for (inside_degrees, expected_pieces) in [(90.0_f32, 2), (30.0, 2), (28.0, 3)] {
        let direction = Vec2::from_angle(std::f32::consts::PI - inside_degrees.to_radians());
        let pieces = tessellate(&[
            point(-100.0, 0.0, 4.0),
            point(0.0, 0.0, 4.0),
            PolylinePoint {
                position: direction * 100.0,
                width: 4.0,
            },
        ]);
        assert_eq!(
            pieces.len(),
            expected_pieces,
            "inside angle {inside_degrees}"
        );
    }
}

#[test]
fn duplicate_points_keep_last_width_and_original_uv_index() {
    let pieces = tessellate(&[
        point(0.0, 0.0, 2.0),
        point(0.0, 0.0, 4.0),
        point(20.0, 0.0, 6.0),
        point(20.0, 0.0, 8.0),
    ]);
    assert_eq!(pieces.len(), 1);
    close(corners(&pieces[0])[0], Vec2::new(0.0, 2.0));
    close(corners(&pieces[0])[1], Vec2::new(20.0, 4.0));
    close(uv(&pieces[0])[0], Vec2::new(1.0 / 3.0, 0.0));
    close(uv(&pieces[0])[2], Vec2::ONE);
}

#[test]
fn empty_runs_reversals_and_zero_widths_remain_finite() {
    assert!(tessellate(&[]).is_empty());
    assert!(tessellate(&[point(0.0, 0.0, 1.0)]).is_empty());
    assert!(tessellate(&[point(0.0, 0.0, 1.0), point(0.0, 0.0, 2.0)]).is_empty());
    for width in [0.0, 4.0] {
        for y in [0.0, 1e-8, 0.01] {
            let pieces = tessellate(&[
                point(-1.0, 0.0, width),
                point(0.0, 0.0, width),
                point(-1.0, y, width),
            ]);
            assert!(!pieces.is_empty());
            for piece in pieces {
                assert!(piece.xform.is_finite());
                assert!(piece.texcoord.is_finite());
                assert!(corners(&piece).iter().all(|p| p.length() < 4.0));
            }
        }
    }
}

#[test]
fn invalid_inputs_are_rejected_even_for_empty_geometry() {
    for invalid in [
        point(0.0, 0.0, -1.0),
        point(0.0, 0.0, f32::NAN),
        point(0.0, 0.0, f32::INFINITY),
        point(f32::NAN, 0.0, 1.0),
    ] {
        assert!(std::panic::catch_unwind(|| tessellate(&[invalid])).is_err());
    }
}
