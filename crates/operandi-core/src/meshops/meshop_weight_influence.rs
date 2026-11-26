//! Weight Influence as a Mesh Stack deformer
//! (`deform.weightInfluence.item`, Operandi's own id): moves each point by
//! Offset times its value in a weight map, so painting the map paints the
//! move.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; points
//! the map has no value for don't move; an empty Map Name moves every
//! point by the whole Offset.
use crate::mesh_stack_builtin::distance;
use crate::meshop_deformer_common::{FOLDER, deform_points, strength, weights};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const WEIGHT_INFLUENCE_ID: &str = "deform.weightInfluence.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WeightInfluenceOp {
    pub map_name: String,
    pub offset_x: f64,
    pub offset_y: f64,
    pub offset_z: f64,
    pub strength: f64,
}
impl Default for WeightInfluenceOp {
    fn default() -> Self {
        Self {
            map_name: "Weight".into(),
            offset_x: 0.,
            offset_y: 0.5,
            offset_z: 0.,
            strength: 1.,
        }
    }
}
impl TypedOp for WeightInfluenceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        crate::mesh_stack_builtin::finite(
            &[self.offset_x, self.offset_y, self.offset_z],
            "Weight Influence Offset",
        )?;
        let map = weights(ctx.input, &self.map_name, "Weight Influence")?;
        let offset = DVec3::new(self.offset_x, self.offset_y, self.offset_z);
        deform_points(ctx, self.strength, |i, p| {
            let w = map.map_or(1., |m| m.get(&(i as u32)).copied().unwrap_or(0.));
            p + offset * w
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<WeightInfluenceOp>::new(
            WEIGHT_INFLUENCE_ID,
            "Weight Influence",
            FOLDER,
            true,
            vec![
                Field::new("mapName", "Map Name", FieldKind::Text),
                distance("offsetX", "Offset X"),
                distance("offsetY", "Offset Y"),
                distance("offsetZ", "Offset Z"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
