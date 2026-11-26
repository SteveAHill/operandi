//! Curve Constraint as a Mesh Stack deformer
//! (`deform.curveConstraint.item`, Operandi's own id): pulls each point
//! onto the nearest spot of one of the mesh's curves, by Strength; with
//! a Radius, only points that near the curve are pulled.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; Radius 0
//! pulls every point; the curve's own points don't move; with no curves
//! the step passes the mesh through.
use crate::mesh_stack_builtin::{count, distance};
use crate::meshop_curve_deform_common::{CurvePath, curve_points, curves_of, pick};
use crate::meshop_deformer_common::{FOLDER, deform_points, strength};
use crate::meshop_types::{OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{CurveKind, Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const CURVE_CONSTRAINT_ID: &str = "deform.curveConstraint.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CurveConstraintOp {
    pub index: u32,
    pub radius: f64,
    pub strength: f64,
}
impl Default for CurveConstraintOp {
    fn default() -> Self {
        Self {
            index: 0,
            radius: 0.,
            strength: 1.,
        }
    }
}
impl TypedOp for CurveConstraintOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        crate::mesh_stack_builtin::finite(&[self.radius], "Curve Constraint Radius")?;
        if self.radius < 0. {
            return Err("Curve Constraint Radius can't be below 0".into());
        }
        let curves = curves_of(ctx.input, |_: CurveKind| true);
        let Some(f) = pick(&curves, self.index, "Curve Constraint")? else {
            return Ok(ctx.input.clone());
        };
        let Some(path) = CurvePath::of(ctx.input, f) else {
            return Ok(ctx.input.clone());
        };
        let fixed = curve_points(ctx.input);
        deform_points(ctx, self.strength, |i, p| {
            let q = path.nearest(p);
            if fixed.contains(&(i as u32)) || (self.radius > 0. && q.distance(p) > self.radius) {
                return p;
            }
            q
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<CurveConstraintOp>::new(
            CURVE_CONSTRAINT_ID,
            "Curve Constraint",
            FOLDER,
            true,
            vec![
                count("index", "Curve Index", 0, 100_000),
                distance("radius", "Radius"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
