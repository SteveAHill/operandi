//! Transform Effector as a Mesh Stack deformer (`deform.transform.item`,
//! Operandi's own id): moves, turns and scales the points about a Center,
//! as one transform: scale first, then turn (X, then Y, then Z), then
//! move by Position.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; a new
//! step starts with no change.
use crate::mesh_stack_builtin::distance;
use crate::meshop_deformer_common::{FOLDER, deform, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DQuat, DVec3, EulerRot};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const TRANSFORM_EFFECTOR_ID: &str = "deform.transform.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TransformEffectorOp {
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    /// Radians.
    pub rot_x: f64,
    pub rot_y: f64,
    pub rot_z: f64,
    pub scl_x: f64,
    pub scl_y: f64,
    pub scl_z: f64,
    pub strength: f64,
}
impl Default for TransformEffectorOp {
    fn default() -> Self {
        Self {
            center_x: 0.,
            center_y: 0.,
            center_z: 0.,
            pos_x: 0.,
            pos_y: 0.,
            pos_z: 0.,
            rot_x: 0.,
            rot_y: 0.,
            rot_z: 0.,
            scl_x: 1.,
            scl_y: 1.,
            scl_z: 1.,
            strength: 1.,
        }
    }
}
impl TypedOp for TransformEffectorOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let v = [
            self.center_x,
            self.center_y,
            self.center_z,
            self.pos_x,
            self.pos_y,
            self.pos_z,
            self.rot_x,
            self.rot_y,
            self.rot_z,
            self.scl_x,
            self.scl_y,
            self.scl_z,
        ];
        crate::mesh_stack_builtin::finite(&v, "Transform Effector values")?;
        let c = DVec3::new(self.center_x, self.center_y, self.center_z);
        let move_by = DVec3::new(self.pos_x, self.pos_y, self.pos_z);
        let turn = DQuat::from_euler(EulerRot::ZYX, self.rot_z, self.rot_y, self.rot_x);
        let scale = DVec3::new(self.scl_x, self.scl_y, self.scl_z);
        deform(ctx, self.strength, |p| {
            c + turn * ((p - c) * scale) + move_by
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let angle = |k, l| Field::new(k, l, FieldKind::Angle);
    let percent = |k, l| Field::new(k, l, FieldKind::Percent);
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<TransformEffectorOp>::new(
            TRANSFORM_EFFECTOR_ID,
            "Transform Effector",
            FOLDER,
            true,
            vec![
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                distance("posX", "Position X"),
                distance("posY", "Position Y"),
                distance("posZ", "Position Z"),
                angle("rotX", "Rotation X"),
                angle("rotY", "Rotation Y"),
                angle("rotZ", "Rotation Z"),
                percent("sclX", "Scale X"),
                percent("sclY", "Scale Y"),
                percent("sclZ", "Scale Z"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
