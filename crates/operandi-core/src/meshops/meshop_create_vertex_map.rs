//! Create Vertex Map as a Mesh Stack op (`setvmap.create.item`,
//! TYPE-IDS.md): adds an empty vertex map called Map Name of the chosen
//! Map Type, for later steps to fill. A map that already exists is left
//! as it is; a blank name adds nothing.
//!
//! Provisional (MOQ-56): the Map Type list past Weight (UV, Morph, RGB,
//! RGBA) and its storage by position; RGB and RGBA maps are both colour
//! maps in Operandi.
use crate::mesh_stack_builtin::choice;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateVertexMapOp {
    #[serde(rename = "MapName")]
    pub map_name: String,
    /// 0 Weight, 1 UV, 2 Morph, 3 RGB, 4 RGBA.
    #[serde(rename = "MapType")]
    pub map_type: u32,
}
impl TypedOp for CreateVertexMapOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.map_type > 4 {
            return Err("Map Type must be 0 (Weight) to 4 (RGBA)".into());
        }
        let mut out = ctx.input.clone();
        let name = self.map_name.trim().to_owned();
        if name.is_empty() {
            return Ok(out);
        }
        match self.map_type {
            0 => {
                out.attributes.point_weights.entry(name).or_default();
            }
            1 => {
                if !out.uv_maps.contains_key(&name) {
                    let rows = crate::uv_maps::empty_rows(&out);
                    out.uv_maps.insert(name, rows);
                }
            }
            2 => {
                out.attributes.relative_morphs.entry(name).or_default();
            }
            _ => {
                out.attributes.point_colors.entry(name).or_default();
            }
        }
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<CreateVertexMapOp>::new(
        "setvmap.create.item",
        "Create Vertex Map",
        "Vertex Map",
        false,
        vec![
            Field::new("MapName", "Map Name", FieldKind::Text),
            choice(
                "MapType",
                "Map Type",
                &["Weight", "UV", "Morph", "RGB", "RGBA"],
            ),
        ],
    ))]
}
