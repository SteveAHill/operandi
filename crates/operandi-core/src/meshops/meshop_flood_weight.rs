//! Flood Weight as a Mesh Stack op (`setvmap.flood.item`, TYPE-IDS.md):
//! every chosen point gets the same weight in the weight map Map Name
//! (made when missing): 100% for High, 0% for Low.
//!
//! Provisional (MOQ-58): High and Low read as 100% and 0%, stored by
//! position; a blank Map Name and Map Type as Set Weight (MOQ-57).
use crate::mesh_stack_builtin::choice;
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::meshop_vertex_map_common::{chosen_points, weight_map_name};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloodWeightOp {
    #[serde(rename = "MapName")]
    pub map_name: String,
    /// 0 High, 1 Low.
    #[serde(rename = "HighOrLow")]
    pub high_or_low: u32,
    #[serde(rename = "UseWeightContainer")]
    pub use_weight_container: u32,
}
impl TypedOp for FloodWeightOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let value = match self.high_or_low {
            0 => 1.,
            1 => 0.,
            _ => return Err("High or Low must be 0 (High) or 1 (Low)".into()),
        };
        let mut out = ctx.input.clone();
        let name = weight_map_name(&out, &self.map_name);
        let map = out.attributes.point_weights.entry(name).or_default();
        for v in chosen_points(ctx) {
            map.insert(v, value);
        }
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<FloodWeightOp>::new(
            "setvmap.flood.item",
            "Flood Weight",
            "Vertex Map",
            true,
            vec![
                Field::new("MapName", "Map Name", FieldKind::Text),
                choice("HighOrLow", "High or Low", &["High", "Low"]),
                hidden("UseWeightContainer"),
            ],
        ))],
        crate::meshop_types::SelElement::Point,
    )
}
