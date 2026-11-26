//! Curve Slice as a Mesh Stack op (`curve.slice.item`; the reference saves
//! a Path Generator + Slice Effector + Path Segment Generator, TYPE-IDS.md):
//! one of the mesh's curves, sampled into Steps straight steps, is cut
//! into the polygons along the cutter's Axis, as the Curve Slice tool does
//! (PS-20..22). Start and End keep part of the curve (fractions of its
//! length) and Slide moves that part along it.
//!
//! Provisional (MOQ-64): Curve Index counts the mesh's own curves in order
//! (Operandi has no curve items for the op to read); the curve itself is
//! never cut and stays. Automatic keeps a step wherever the curve has
//! turned by Smoothing Angle, as the tool's Auto does. Action Axis is
//! world Y. With no curves the op passes its input through. The path
//! generator's alignment channels only matter for clones and aren't kept.
use crate::mesh_stack_builtin::{check, count};
use crate::meshop_curve_common::{kind_of, points_of};
use crate::meshop_cutter_common::{cutter_fields, cutter_op};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::path_slice::CutPath;
use crate::{Mesh, Result};
use glam::DVec3;
use std::sync::Arc;

cutter_op! {
    pub struct CurveSliceOp {
        /// Curve Index: which of the mesh's curves, counting from 0.
        index: u32 = 0,
        /// Fractions of the curve's length.
        slide: f64 = 0.,
        start: f64 = 0.,
        end: f64 = 1.,
        /// Steps.
        number: u32 = 24,
        auto: bool = false,
        /// Smoothing Angle, radians.
        smooth: f64 = 0.087,
    }
    axis = 1;
}

/// The mesh's curve elements, in order.
pub(crate) fn curve_elements(mesh: &Mesh) -> Vec<usize> {
    (0..mesh.faces.len())
        .filter(|&f| mesh.is_polyline(f))
        .collect()
}

/// Curve `index`, or `None` when the mesh has no curves.
pub(crate) fn pick_curve(mesh: &Mesh, index: u32, what: &str) -> Result<Option<usize>> {
    let curves = curve_elements(mesh);
    if curves.is_empty() {
        return Ok(None);
    }
    curves
        .get(index as usize)
        .copied()
        .map(Some)
        .ok_or_else(|| {
            format!(
                "{what} {index}: the mesh has {} curve{} (0 to {})",
                curves.len(),
                if curves.len() == 1 { "" } else { "s" },
                curves.len() - 1
            )
        })
}

/// The point `t` (0..=1) of the way along `line` by length.
fn at_length(line: &[DVec3], lengths: &[f64], t: f64) -> DVec3 {
    let total = *lengths.last().unwrap_or(&0.);
    let want = t.clamp(0., 1.) * total;
    let k = lengths
        .partition_point(|&l| l < want)
        .clamp(1, line.len() - 1);
    let (l0, l1) = (lengths[k - 1], lengths[k]);
    let f = if l1 > l0 { (want - l0) / (l1 - l0) } else { 0. };
    line[k - 1].lerp(line[k], f)
}

impl CurveSliceOp {
    /// The cut path: the kept part of curve `f`, in steps.
    fn path(&self, mesh: &Mesh, f: usize) -> Result<CutPath> {
        crate::mesh_stack_builtin::finite(
            &[self.slide, self.start, self.end, self.smooth],
            "Curve Slice values",
        )?;
        if !(1..=4096).contains(&self.number) {
            return Err("Steps must be 1 to 4096".into());
        }
        let (kind, closed) = kind_of(mesh, f);
        let points = points_of(mesh, f);
        let mut line = crate::curves::sample(&points, kind, closed, 32);
        if closed && line.len() > 1 && line[0].distance(*line.last().unwrap()) > 1e-12 {
            line.push(line[0]);
        }
        let mut lengths = vec![0.];
        for w in line.windows(2) {
            lengths.push(lengths.last().unwrap() + w[0].distance(w[1]));
        }
        let (a, b) = (
            (self.start + self.slide).clamp(0., 1.),
            (self.end + self.slide).clamp(0., 1.),
        );
        if line.len() < 2 || b - a <= 1e-9 || *lengths.last().unwrap() <= 1e-12 {
            return Ok(CutPath {
                points: Vec::new(),
                closed: false,
            });
        }
        // The whole of a closed curve stays a closed outline.
        let whole_loop = closed && a <= 1e-9 && b >= 1. - 1e-9;
        let points: Vec<DVec3> = if self.auto {
            let total = *lengths.last().unwrap();
            let mut dense = vec![at_length(&line, &lengths, a)];
            dense.extend(
                line.iter()
                    .zip(&lengths)
                    .filter(|(_, l)| **l > a * total + 1e-12 && **l < b * total - 1e-12)
                    .map(|(p, _)| *p),
            );
            dense.push(at_length(&line, &lengths, b));
            crate::path_slice::by_angle(&dense, false, self.smooth.to_degrees())
        } else {
            let steps = self.number as usize;
            (0..=steps)
                .map(|i| at_length(&line, &lengths, a + (b - a) * i as f64 / steps as f64))
                .collect()
        };
        let mut points = points;
        if whole_loop && points.len() > 2 {
            points.pop();
        }
        Ok(CutPath {
            points,
            closed: whole_loop,
        })
    }
}

impl TypedOp for CurveSliceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let cutter = self.cutter()?;
        let Some(f) = pick_curve(ctx.input, self.index, "Curve Index")? else {
            return Ok(ctx.input.clone());
        };
        let path = self.path(ctx.input, f)?;
        cutter.cut(ctx, &[path], DVec3::Y)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let mut fields = vec![
        count("index", "Curve Index", 0, 100_000),
        Field::new("slide", "Slide", FieldKind::Percent),
        Field::new("start", "Start", FieldKind::Percent),
        Field::new("end", "End", FieldKind::Percent),
        count("number", "Steps", 1, 4096),
        check("auto", "Automatic"),
        Field::new("smooth", "Smoothing Angle", FieldKind::Angle),
    ];
    fields.extend(cutter_fields());
    vec![Arc::new(Typed::<CurveSliceOp>::new(
        "curve.slice.item",
        "Curve Slice",
        "Edit",
        true,
        fields,
    ))]
}
