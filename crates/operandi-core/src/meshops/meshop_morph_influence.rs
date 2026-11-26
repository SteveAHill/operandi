//! Morph Influence as a Mesh Stack deformer
//! (`deform.morphInfluence.item`, Operandi's own id): moves each point by
//! its offset in a morph map, times Strength, so a morph target can be
//! dialled in partway (or past 100%) in the stack.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; only
//! relative morph maps (offsets) are read; points the map doesn't list
//! don't move.
use crate::meshop_deformer_common::{FOLDER, deform_points};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const MORPH_INFLUENCE_ID: &str = "deform.morphInfluence.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MorphInfluenceOp {
    pub map_name: String,
    /// Can go past 1 or below 0, unlike the other deformers' Strength.
    pub strength: f64,
}
impl Default for MorphInfluenceOp {
    fn default() -> Self {
        Self {
            map_name: "Morph".into(),
            strength: 1.,
        }
    }
}
impl TypedOp for MorphInfluenceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let Some(morph) = ctx.input.attributes.relative_morphs.get(&self.map_name) else {
            return Err(format!(
                "Morph Influence: the mesh has no morph map \u{201c}{}\u{201d}",
                self.map_name
            ));
        };
        deform_points(ctx, self.strength, |i, p| {
            p + morph.get(&(i as u32)).copied().unwrap_or(DVec3::ZERO)
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<MorphInfluenceOp>::new(
            MORPH_INFLUENCE_ID,
            "Morph Influence",
            FOLDER,
            true,
            vec![
                Field::new("mapName", "Map Name", FieldKind::Text),
                crate::meshop_deformer_common::strength(),
            ],
        ))],
        SelElement::Point,
    )
}
