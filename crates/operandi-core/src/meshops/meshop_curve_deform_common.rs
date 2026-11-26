//! What the curve deformers in the Mesh Stack share (Spline Deformer,
//! Bezier Deformer, Curve Constraint; deformers plan batch 13): one of the
//! mesh's own curves, sampled into a polyline with a frame at each step
//! that turns as little as it can (parallel transport). The curve's own
//! points never move.
use crate::meshop_curve_common::{kind_of, points_of};
use crate::{CurveKind, Mesh};
use glam::{DQuat, DVec3};
use std::collections::BTreeSet;

/// A sampled curve: points, length so far at each, and each span's
/// tangent and normal.
pub(crate) struct CurvePath {
    pub points: Vec<DVec3>,
    pub lengths: Vec<f64>,
    pub tangents: Vec<DVec3>,
    pub normals: Vec<DVec3>,
}

/// The world axis least in line with `t`, made square to it.
pub(crate) fn square_to(t: DVec3) -> DVec3 {
    let a = DVec3::AXES
        .into_iter()
        .min_by(|a, b| a.dot(t).abs().total_cmp(&b.dot(t).abs()))
        .unwrap_or(DVec3::X);
    (a - t * a.dot(t)).normalize_or(DVec3::X)
}

impl CurvePath {
    /// Curve element `f` of `mesh`, or `None` when it has no length.
    pub(crate) fn of(mesh: &Mesh, f: usize) -> Option<Self> {
        let (kind, closed) = kind_of(mesh, f);
        let mut points = crate::curves::sample(&points_of(mesh, f), kind, closed, 32);
        if closed && points.len() > 1 {
            points.push(points[0]);
        }
        points.dedup_by(|a, b| a.distance(*b) < 1e-12);
        if points.len() < 2 {
            return None;
        }
        let mut lengths = vec![0.];
        let mut tangents = Vec::new();
        for w in points.windows(2) {
            lengths.push(lengths.last().unwrap() + w[0].distance(w[1]));
            tangents.push((w[1] - w[0]).normalize());
        }
        let mut normals = vec![square_to(tangents[0])];
        for w in tangents.windows(2) {
            let turn = DQuat::from_rotation_arc(w[0], w[1]);
            let n = turn * *normals.last().unwrap();
            normals.push((n - w[1] * n.dot(w[1])).normalize_or(square_to(w[1])));
        }
        Some(Self {
            points,
            lengths,
            tangents,
            normals,
        })
    }
    /// The place `s` along the curve (past either end, straight on) and
    /// the frame there: tangent, normal, binormal.
    pub(crate) fn at(&self, s: f64) -> (DVec3, [DVec3; 3]) {
        let spans = self.tangents.len();
        let k = self.lengths.partition_point(|&l| l <= s).clamp(1, spans) - 1;
        let (t, n) = (self.tangents[k], self.normals[k]);
        let p = self.points[k] + t * (s - self.lengths[k]);
        (p, [t, n, t.cross(n)])
    }
    /// The nearest spot on the curve to `p`.
    pub(crate) fn nearest(&self, p: DVec3) -> DVec3 {
        self.points
            .windows(2)
            .map(|w| {
                let d = w[1] - w[0];
                let t = ((p - w[0]).dot(d) / d.length_squared()).clamp(0., 1.);
                w[0] + d * t
            })
            .min_by(|a, b| a.distance_squared(p).total_cmp(&b.distance_squared(p)))
            .unwrap_or(p)
    }
}

/// The mesh's curve elements of the kinds `keep` allows, in order.
pub(crate) fn curves_of(mesh: &Mesh, keep: impl Fn(CurveKind) -> bool) -> Vec<usize> {
    crate::meshop_curve_slice::curve_elements(mesh)
        .into_iter()
        .filter(|&f| keep(kind_of(mesh, f).0))
        .collect()
}

/// Every point some curve element uses (they don't move).
pub(crate) fn curve_points(mesh: &Mesh) -> BTreeSet<u32> {
    crate::meshop_curve_slice::curve_elements(mesh)
        .into_iter()
        .flat_map(|f| mesh.faces[f].vertices.iter().copied())
        .collect()
}

/// Curve `index` of `curves`, `None` with no curves, an error past the end.
pub(crate) fn pick(curves: &[usize], index: u32, what: &str) -> crate::Result<Option<usize>> {
    if curves.is_empty() {
        return Ok(None);
    }
    curves
        .get(index as usize)
        .copied()
        .map(Some)
        .ok_or_else(|| {
            format!(
                "{what} Curve Index {index}: the mesh has {} such curve{} (0 to {})",
                curves.len(),
                if curves.len() == 1 { "" } else { "s" },
                curves.len() - 1
            )
        })
}
