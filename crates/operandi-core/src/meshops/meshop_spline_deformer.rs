//! Spline Deformer as a Mesh Stack deformer (`deform.spline.item`,
//! Operandi's own id): bends the mesh along one of its curves. The
//! distance along Axis (from Start, times Length Scale) becomes the
//! distance along the curve, and the rest of each point's place is laid
//! square to the curve there, so a straight column follows the curve.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; Curve
//! Index counts the mesh's own curves (any kind); the curve's own points
//! don't move; past the curve's ends the mesh carries on straight. With
//! no curves the step passes the mesh through.
use crate::mesh_stack_builtin::{choice, count, distance};
use crate::meshop_curve_deform_common::{CurvePath, curve_points, curves_of, pick, square_to};
use crate::meshop_deformer_common::{FOLDER, axis_of, deform_points, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{CurveKind, Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const SPLINE_DEFORMER_ID: &str = "deform.spline.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SplineDeformerOp {
    /// Which of the mesh's curves, counting from 0.
    pub index: u32,
    /// 0 X, 1 Y, 2 Z: the mesh's length runs along it.
    pub axis: u32,
    /// Where along Axis the curve starts.
    pub start: f64,
    pub scale: f64,
    pub strength: f64,
}
impl Default for SplineDeformerOp {
    fn default() -> Self {
        Self {
            index: 0,
            axis: 1,
            start: 0.,
            scale: 1.,
            strength: 1.,
        }
    }
}

/// Bends `ctx`'s points along `path` (the shared engine).
pub(crate) fn along(
    ctx: &OpContext,
    path: &CurvePath,
    axis: u32,
    start: f64,
    scale: f64,
    strength: f64,
    what: &str,
) -> Result<Mesh> {
    let a = axis_of(axis, what)?;
    crate::mesh_stack_builtin::finite(&[start, scale], what)?;
    let u = square_to(a);
    let v = a.cross(u);
    let fixed = curve_points(ctx.input);
    deform_points(ctx, strength, |i, p| {
        if fixed.contains(&(i as u32)) {
            return p;
        }
        let (at, [_, n, b]) = path.at((p.dot(a) - start) * scale);
        at + n * p.dot(u) + b * p.dot(v)
    })
}

impl TypedOp for SplineDeformerOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let curves = curves_of(ctx.input, |_: CurveKind| true);
        let Some(f) = pick(&curves, self.index, "Spline Deformer")? else {
            return Ok(ctx.input.clone());
        };
        let Some(path) = CurvePath::of(ctx.input, f) else {
            return Ok(ctx.input.clone());
        };
        along(
            ctx,
            &path,
            self.axis,
            self.start,
            self.scale,
            self.strength,
            "Spline Deformer",
        )
    }
}

pub(crate) fn fields() -> Vec<Field> {
    vec![
        count("index", "Curve Index", 0, 100_000),
        choice("axis", "Axis", &["X", "Y", "Z"]),
        distance("start", "Start"),
        Field::new("scale", "Length Scale", FieldKind::Percent),
        strength(),
    ]
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<SplineDeformerOp>::new(
            SPLINE_DEFORMER_ID,
            "Spline Deformer",
            FOLDER,
            true,
            fields(),
        ))],
        SelElement::Point,
    )
}
