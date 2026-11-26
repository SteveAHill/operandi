//! Push Influence as a Mesh Stack deformer (`deform.pushInfluence.item`,
//! Operandi's own id): pushes each point along its normal (the average of
//! its polygons' normals) by Distance, times its value in a weight map
//! when Map Name names one.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; an empty
//! Map Name pushes every point by the whole Distance; points on no
//! polygon don't move.
use crate::mesh_stack_builtin::distance;
use crate::meshop_deformer_common::{FOLDER, deform_points, strength, weights};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const PUSH_INFLUENCE_ID: &str = "deform.pushInfluence.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PushInfluenceOp {
    pub map_name: String,
    pub distance: f64,
    pub strength: f64,
}
impl Default for PushInfluenceOp {
    fn default() -> Self {
        Self {
            map_name: String::new(),
            distance: 0.1,
            strength: 1.,
        }
    }
}
impl TypedOp for PushInfluenceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        crate::mesh_stack_builtin::finite(&[self.distance], "Push Influence Distance")?;
        let map = weights(ctx.input, &self.map_name, "Push Influence")?;
        let normals = crate::generator_ops::point_normals(ctx.input);
        deform_points(ctx, self.strength, |i, p| {
            let w = map.map_or(1., |m| m.get(&(i as u32)).copied().unwrap_or(0.));
            p + normals[i] * self.distance * w
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<PushInfluenceOp>::new(
            PUSH_INFLUENCE_ID,
            "Push Influence",
            FOLDER,
            true,
            vec![
                Field::new("mapName", "Map Name", FieldKind::Text),
                distance("distance", "Distance"),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
