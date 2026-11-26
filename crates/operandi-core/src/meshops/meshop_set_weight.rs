//! Set Weight as a Mesh Stack op (`setvmap.weight.item`, TYPE-IDS.md): the
//! chosen points get Map Value in the weight map Map Name (made when
//! missing), or, with Additive, Map Value added to what they have.
//!
//! Provisional (MOQ-57): a blank Map Name means the mesh's first weight
//! map ("Weight" when it has none); Map Type is kept, not shown (only
//! plain weight maps are built). Map Value is a fraction (1 = 100%).
use crate::mesh_stack_builtin::check;
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::meshop_vertex_map_common::{chosen_points, weight_map_name};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetWeightOp {
    #[serde(rename = "MapName")]
    pub map_name: String,
    #[serde(rename = "Additive")]
    pub additive: bool,
    #[serde(rename = "MapValue")]
    pub map_value: f64,
    #[serde(rename = "UseWeightContainer")]
    pub use_weight_container: u32,
}
impl Default for SetWeightOp {
    fn default() -> Self {
        Self {
            map_name: String::new(),
            additive: false,
            map_value: 1.,
            use_weight_container: 0,
        }
    }
}
impl TypedOp for SetWeightOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if !self.map_value.is_finite() {
            return Err("Map Value must be a number".into());
        }
        let mut out = ctx.input.clone();
        let name = weight_map_name(&out, &self.map_name);
        let map = out.attributes.point_weights.entry(name).or_default();
        for v in chosen_points(ctx) {
            let w = map.entry(v).or_insert(0.);
            *w = if self.additive {
                *w + self.map_value
            } else {
                self.map_value
            };
        }
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<SetWeightOp>::new(
            "setvmap.weight.item",
            "Set Weight",
            "Vertex Map",
            true,
            vec![
                Field::new("MapName", "Map Name", FieldKind::Text),
                Field::new("MapValue", "Map Value", FieldKind::Percent),
                check("Additive", "Additive"),
                hidden("UseWeightContainer"),
            ],
        ))],
        crate::meshop_types::SelElement::Point,
    )
}
