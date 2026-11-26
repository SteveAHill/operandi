//! Magnet as a Mesh Stack deformer (`deform.magnet.item`, Operandi's own
//! id): pulls the points near the Center by Offset, fully at the Center
//! and fading smoothly to nothing at Radius, as Soft Move does.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; the fade
//! is (1 - (d / Radius)²)²; a new step pulls 0.5 m up within 1 m.
use crate::mesh_stack_builtin::distance;
use crate::meshop_deformer_common::{FOLDER, deform, strength};
use crate::meshop_types::{OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const MAGNET_ID: &str = "deform.magnet.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MagnetOp {
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub radius: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub offset_z: f64,
    pub strength: f64,
}
impl Default for MagnetOp {
    fn default() -> Self {
        Self {
            center_x: 0.,
            center_y: 0.,
            center_z: 0.,
            radius: 1.,
            offset_x: 0.,
            offset_y: 0.5,
            offset_z: 0.,
            strength: 1.,
        }
    }
}
/// Full at the centre, nothing at `radius` and beyond.
pub(crate) fn magnet_weight(d: f64, radius: f64) -> f64 {
    let t = (d / radius).clamp(0., 1.);
    (1. - t * t).powi(2)
}
impl TypedOp for MagnetOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        crate::mesh_stack_builtin::finite(
            &[
                self.center_x,
                self.center_y,
                self.center_z,
                self.radius,
                self.offset_x,
                self.offset_y,
                self.offset_z,
            ],
            "Magnet values",
        )?;
        if self.radius <= 1e-9 {
            return Err("Magnet Radius must be more than 0".into());
        }
        let c = DVec3::new(self.center_x, self.center_y, self.center_z);
        let offset = DVec3::new(self.offset_x, self.offset_y, self.offset_z);
        deform(ctx, self.strength, |p| {
            p + offset * magnet_weight(p.distance(c), self.radius)
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<MagnetOp>::new(
            MAGNET_ID,
            "Magnet",
            FOLDER,
            true,
            vec![
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                distance("radius", "Radius"),
                distance("offsetX", "Offset X"),
                distance("offsetY", "Offset Y"),
                distance("offsetZ", "Offset Z"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
