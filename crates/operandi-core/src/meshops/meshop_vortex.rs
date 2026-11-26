//! Vortex as a Mesh Stack deformer (`deform.vortex.item`, Operandi's own
//! id): swirls the points around a line through the Center along Axis:
//! points on the line turn by the full Angle, and the turn fades smoothly
//! to nothing at Radius from it, as Swirl does.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; the fade
//! is (1 - (d / Radius)²)² on the distance d from the line; Angle starts
//! at 90°.
use crate::mesh_stack_builtin::{choice, distance};
use crate::meshop_deformer_common::{FOLDER, axis_of, deform, strength};
use crate::meshop_magnet::magnet_weight;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DQuat, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const VORTEX_ID: &str = "deform.vortex.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VortexOp {
    /// Radians on the line.
    pub angle: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
    pub radius: f64,
    pub strength: f64,
}
impl Default for VortexOp {
    fn default() -> Self {
        Self {
            angle: 90f64.to_radians(),
            center_x: 0.,
            center_y: 0.,
            center_z: 0.,
            axis: 1,
            radius: 1.,
            strength: 1.,
        }
    }
}
impl TypedOp for VortexOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let axis = axis_of(self.axis, "Vortex")?;
        crate::mesh_stack_builtin::finite(
            &[
                self.angle,
                self.center_x,
                self.center_y,
                self.center_z,
                self.radius,
            ],
            "Vortex values",
        )?;
        if self.radius <= 1e-9 {
            return Err("Vortex Radius must be more than 0".into());
        }
        let c = DVec3::new(self.center_x, self.center_y, self.center_z);
        deform(ctx, self.strength, |p| {
            let rel = p - c;
            let d = (rel - axis * rel.dot(axis)).length();
            let turn = self.angle * magnet_weight(d, self.radius);
            c + DQuat::from_axis_angle(axis, turn) * rel
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<VortexOp>::new(
            VORTEX_ID,
            "Vortex",
            FOLDER,
            true,
            vec![
                Field::new("angle", "Angle", FieldKind::Angle),
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                choice("axis", "Axis", &["X", "Y", "Z"]),
                distance("radius", "Radius"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
