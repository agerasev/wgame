use glam::{Affine2, Affine3A, DVec2, Vec2};

use super::PolylinePoint;
use crate::polygon::quad_transform;

pub(super) struct Piece {
    pub xform: Affine3A,
    pub texcoord: Affine2,
}

impl Piece {
    fn new(corners: [DVec2; 4], texcoord: Affine2) -> Self {
        Self {
            xform: quad_transform(corners.map(|p| p.as_vec2().extend(0.0))),
            texcoord,
        }
    }
}

struct Point {
    position: DVec2,
    radius: f64,
    u: f32,
}

struct Segment {
    direction: DVec2,
    length: f64,
}

struct Join {
    incoming: [DVec2; 2],
    outgoing: [DVec2; 2],
    bevel: Option<Piece>,
}

impl Join {
    fn shared(edge: [DVec2; 2]) -> Self {
        Self {
            incoming: edge,
            outgoing: edge,
            bevel: None,
        }
    }
}

fn edge(point: &Point, offset: DVec2) -> [DVec2; 2] {
    [point.position + offset, point.position - offset]
}

fn join(point: &Point, before: &Segment, after: &Segment) -> Join {
    let before_normal = before.direction.perp();
    let after_normal = after.direction.perp();
    let bisector = (before_normal + after_normal).normalize_or_zero();
    let cosine = bisector.dot(after_normal);
    if cosine <= 1e-6 {
        // A reversal has no finite offset-line intersection. Split the runs
        // rather than constructing an enormous or non-finite join.
        return Join {
            incoming: edge(point, before_normal * point.radius),
            outgoing: edge(point, after_normal * point.radius),
            bevel: None,
        };
    }
    let offset = bisector * (point.radius / cosine);
    if cosine >= 0.25 || point.radius == 0.0 {
        return Join::shared(edge(point, offset));
    }

    let turn = before.direction.perp_dot(after.direction).signum();
    let inner_offset = turn * offset;
    let inner = if inner_offset.dot(before.direction).abs() <= before.length
        && inner_offset.dot(after.direction).abs() <= after.length
    {
        point.position + inner_offset
    } else {
        // The intersection is outside the neighboring segments. Retaining it
        // would extend the ribbon past its control points and fold the quads.
        point.position
    };
    let outer_before = point.position - turn * before_normal * point.radius;
    let outer_after = point.position - turn * after_normal * point.radius;
    let (incoming, outgoing, outer_v) = if turn > 0.0 {
        ([inner, outer_before], [inner, outer_after], 1.0)
    } else {
        ([outer_before, inner], [outer_after, inner], 0.0)
    };

    Join {
        incoming,
        outgoing,
        // Collapse one edge to encode the bevel triangle in the same shared
        // mesh as the segment quads. Its U is constant across the junction.
        bevel: Some(Piece::new(
            [outer_before, inner, inner, outer_after],
            Affine2::from_cols(
                Vec2::new(0.0, 1.0 - 2.0 * outer_v),
                Vec2::ZERO,
                Vec2::new(point.u, outer_v),
            ),
        )),
    }
}

pub(super) fn tessellate(input: &[PolylinePoint]) -> Vec<Piece> {
    let mut points: Vec<Point> = Vec::with_capacity(input.len());
    for (i, point) in input.iter().enumerate() {
        assert!(
            point.position.is_finite(),
            "polyline positions must be finite"
        );
        assert!(
            point.width.is_finite() && point.width >= 0.0,
            "polyline widths must be finite and nonnegative"
        );
        let point = Point {
            position: point.position.as_dvec2(),
            radius: 0.5 * f64::from(point.width),
            u: i as f32 / input.len().saturating_sub(1).max(1) as f32,
        };
        if points
            .last()
            .is_some_and(|last| last.position == point.position)
        {
            *points.last_mut().unwrap() = point;
        } else {
            points.push(point);
        }
    }
    if points.len() < 2 {
        return Vec::new();
    }
    // f64 intermediates keep subtraction and normalization finite even when
    // valid f32 inputs are far apart or extremely close together.
    let segments: Vec<_> = points
        .windows(2)
        .map(|pair| {
            let delta = pair[1].position - pair[0].position;
            let length = delta.length();
            Segment {
                direction: delta / length,
                length,
            }
        })
        .collect();
    let mut joins: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, point)| match (i.checked_sub(1), segments.get(i)) {
            (None, Some(after)) => Join::shared(edge(point, after.direction.perp() * point.radius)),
            (Some(before), None) => Join::shared(edge(
                point,
                segments[before].direction.perp() * point.radius,
            )),
            (Some(before), Some(after)) => join(point, &segments[before], after),
            (None, None) => unreachable!("at least one segment"),
        })
        .collect();
    let mut pieces = Vec::with_capacity(2 * segments.len());
    for i in 0..segments.len() {
        let [left_start, right_start] = joins[i].outgoing;
        let [left_end, right_end] = joins[i + 1].incoming;
        pieces.push(Piece::new(
            [left_start, left_end, right_end, right_start],
            Affine2::from_cols(
                Vec2::new(points[i + 1].u - points[i].u, 0.0),
                Vec2::Y,
                Vec2::new(points[i].u, 0.0),
            ),
        ));
        if let Some(bevel) = joins[i + 1].bevel.take() {
            pieces.push(bevel);
        }
    }
    pieces
}

#[cfg(test)]
mod tests;
