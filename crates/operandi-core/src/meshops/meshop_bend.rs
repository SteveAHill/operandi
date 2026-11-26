//! Bend as a Mesh Stack deformer (`deform.bend.item`, Operandi's own id):
//! curls the points into an arc with the Bend tool's kernel (bend.rs):
//! points further along the Spine from the Center turn further, up to the
//! full Angle at its end, around the chosen Axis.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; Angle
//! starts at 90° so a new step shows its effect. An Axis along the Spine
//! can't turn it, and the step then leaves the points where they are.
use crate::bend::BendOptions;
use crate::mesh_stack_builtin::{check, choice, distance};
use crate::meshop_deformer_common::{FOLDER, deform, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DMat3, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const BEND_ID: &str = "deform.bend.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BendOp {
    /// Radians.
    pub angle: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub spine_x: f64,
    pub spine_y: f64,
    pub spine_z: f64,
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
    pub both_sides: bool,
    pub strength: f64,
}
impl Default for BendOp {
    fn default() -> Self {
        let o = BendOptions::default();
        Self {
            angle: 90f64.to_radians(),
            center_x: 0.,
            center_y: 0.,
            center_z: 0.,
            spine_x: o.spine.x,
            spine_y: o.spine.y,
            spine_z: o.spine.z,
            axis: o.axis as u32,
            both_sides: o.both_sides,
            strength: 1.,
        }
    }
}
impl TypedOp for BendOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.axis > 2 {
            return Err("Bend Axis must be 0 (X), 1 (Y) or 2 (Z)".into());
        }
        let o = BendOptions {
            angle: self.angle.to_degrees(),
            base: DVec3::new(self.center_x, self.center_y, self.center_z),
            spine: DVec3::new(self.spine_x, self.spine_y, self.spine_z),
            both_sides: self.both_sides,
            axis: self.axis as usize,
            deform_space: true,
        };
        o.validate()?;
        if o.axis_along_spine(DMat3::IDENTITY) {
            return Ok(ctx.input.clone());
        }
        let direction = o.direction(DMat3::IDENTITY);
        deform(ctx, self.strength, |p| o.bend_point(p, direction))
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<BendOp>::new(
            BEND_ID,
            "Bend",
            FOLDER,
            true,
            vec![
                Field::new("angle", "Angle", FieldKind::Angle),
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                distance("spineX", "Spine X"),
                distance("spineY", "Spine Y"),
                distance("spineZ", "Spine Z"),
                choice("axis", "Axis", &["X", "Y", "Z"]),
                check("bothSides", "Both Sides"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
