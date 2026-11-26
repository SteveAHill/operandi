//! Rotate Effector as a Mesh Stack deformer (`deform.rotate.item`,
//! Operandi's own id): turns the points by Angle about a line through the
//! Center along Axis.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; Angle
//! starts at 90° so a new step shows its effect.
use crate::mesh_stack_builtin::{choice, distance};
use crate::meshop_deformer_common::{FOLDER, axis_of, deform, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DQuat, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const ROTATE_EFFECTOR_ID: &str = "deform.rotate.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RotateEffectorOp {
    /// Radians.
    pub angle: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
    pub strength: f64,
}
impl Default for RotateEffectorOp {
    fn default() -> Self {
        Self {
            angle: 90f64.to_radians(),
            center_x: 0.,
            center_y: 0.,
            center_z: 0.,
            axis: 1,
            strength: 1.,
        }
    }
}
impl TypedOp for RotateEffectorOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let axis = axis_of(self.axis, "Rotate Effector")?;
        crate::mesh_stack_builtin::finite(
            &[self.angle, self.center_x, self.center_y, self.center_z],
            "Rotate Effector values",
        )?;
        let c = DVec3::new(self.center_x, self.center_y, self.center_z);
        let turn = DQuat::from_axis_angle(axis, self.angle);
        deform(ctx, self.strength, |p| c + turn * (p - c))
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<RotateEffectorOp>::new(
            ROTATE_EFFECTOR_ID,
            "Rotate Effector",
            FOLDER,
            true,
            vec![
                Field::new("angle", "Angle", FieldKind::Angle),
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                choice("axis", "Axis", &["X", "Y", "Z"]),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
