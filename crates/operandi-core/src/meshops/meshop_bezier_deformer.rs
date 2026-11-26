//! Bezier Deformer as a Mesh Stack deformer (`deform.bezier.item`,
//! Operandi's own id): the Spline Deformer's bend, along one of the
//! mesh's Bezier curves (Curve Index counts only those).
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; with no
//! Bezier curves the step passes the mesh through.
use crate::meshop_curve_deform_common::{CurvePath, curves_of, pick};
use crate::meshop_deformer_common::FOLDER;
use crate::meshop_spline_deformer::{SplineDeformerOp, along, fields};
use crate::meshop_types::{OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{CurveKind, Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const BEZIER_DEFORMER_ID: &str = "deform.bezier.item";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BezierDeformerOp(pub SplineDeformerOp);

impl TypedOp for BezierDeformerOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let o = &self.0;
        let curves = curves_of(ctx.input, |k| k == CurveKind::Bezier);
        let Some(f) = pick(&curves, o.index, "Bezier Deformer")? else {
            return Ok(ctx.input.clone());
        };
        let Some(path) = CurvePath::of(ctx.input, f) else {
            return Ok(ctx.input.clone());
        };
        along(
            ctx,
            &path,
            o.axis,
            o.start,
            o.scale,
            o.strength,
            "Bezier Deformer",
        )
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<BezierDeformerOp>::new(
            BEZIER_DEFORMER_ID,
            "Bezier Deformer",
            FOLDER,
            true,
            fields(),
        ))],
        SelElement::Point,
    )
}
