//! Pen Slice as a Mesh Stack op (`pen.slice.item`; the reference saves a
//! Pen Generator + Slice Effector, TYPE-IDS.md): one of the mesh's curves,
//! taken as straight segments between its points, is cut into the
//! polygons along the cutter's Axis, as the Pen Slice tool cuts its
//! clicked points (PS-30/31). A closed curve cuts a closed outline.
//!
//! Provisional (MOQ-64): Polyline Index counts the mesh's own curves in
//! order, as Curve Slice's Curve Index does; the curve is never cut and
//! stays. Action Axis is world Y. With no curves the op passes its input
//! through. The pen generator's alignment and corner channels only matter
//! for clones and aren't kept.
use crate::mesh_stack_builtin::count;
use crate::meshop_curve_common::{kind_of, points_of};
use crate::meshop_curve_slice::pick_curve;
use crate::meshop_cutter_common::{cutter_fields, cutter_op};
use crate::meshop_types::{OpContext, OpDef, Typed, TypedOp};
use crate::path_slice::CutPath;
use crate::{Mesh, Result};
use glam::DVec3;
use std::sync::Arc;

cutter_op! {
    pub struct PenSliceOp {
        /// Polyline Index: which of the mesh's curves, counting from 0.
        index: u32 = 0,
    }
    axis = 1;
}

impl TypedOp for PenSliceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let cutter = self.cutter()?;
        let Some(f) = pick_curve(ctx.input, self.index, "Polyline Index")? else {
            return Ok(ctx.input.clone());
        };
        let (_, closed) = kind_of(ctx.input, f);
        let path = CutPath {
            points: points_of(ctx.input, f),
            closed,
        };
        cutter.cut(ctx, &[path], DVec3::Y)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let mut fields = vec![count("index", "Polyline Index", 0, 100_000)];
    fields.extend(cutter_fields());
    vec![Arc::new(Typed::<PenSliceOp>::new(
        "pen.slice.item",
        "Pen Slice",
        "Edit",
        true,
        fields,
    ))]
}
