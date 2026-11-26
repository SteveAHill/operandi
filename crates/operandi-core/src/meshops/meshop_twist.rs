//! Twist as a Mesh Stack deformer (`deform.twist.item`, Operandi's own
//! id): turns the points about a line through the Center along the Axis,
//! by an angle that grows with their distance along it: none at the
//! Center, the full Angle at Length and beyond, as the Twist tool does
//! (each point turns by its share of the Angle, TW-Q1).
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; Angle
//! starts at 90° so a new step shows its effect. Points behind the Center
//! don't turn.
use crate::mesh_stack_builtin::{choice, distance};
use crate::meshop_deformer_common::{FOLDER, deform, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DQuat, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const TWIST_ID: &str = "deform.twist.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TwistOp {
    /// Radians at Length.
    pub angle: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
    pub length: f64,
    pub strength: f64,
}
impl Default for TwistOp {
    fn default() -> Self {
        Self {
            angle: 90f64.to_radians(),
            center_x: 0.,
            center_y: 0.,
            center_z: 0.,
            axis: 1,
            length: 1.,
            strength: 1.,
        }
    }
}
impl TypedOp for TwistOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let axis = match self.axis {
            0 => DVec3::X,
            1 => DVec3::Y,
            2 => DVec3::Z,
            _ => return Err("Twist Axis must be 0 (X), 1 (Y) or 2 (Z)".into()),
        };
        crate::mesh_stack_builtin::finite(
            &[
                self.angle,
                self.center_x,
                self.center_y,
                self.center_z,
                self.length,
            ],
            "Twist values",
        )?;
        if self.length <= 1e-9 {
            return Err("Twist Length must be more than 0".into());
        }
        let c = DVec3::new(self.center_x, self.center_y, self.center_z);
        deform(ctx, self.strength, |p| {
            let t = ((p - c).dot(axis) / self.length).clamp(0., 1.);
            c + DQuat::from_axis_angle(axis, self.angle * t) * (p - c)
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<TwistOp>::new(
            TWIST_ID,
            "Twist",
            FOLDER,
            true,
            vec![
                Field::new("angle", "Angle", FieldKind::Angle),
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                choice("axis", "Axis", &["X", "Y", "Z"]),
                distance("length", "Length"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
