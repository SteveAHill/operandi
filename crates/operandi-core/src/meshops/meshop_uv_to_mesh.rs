//! Convert UV to Mesh as a Mesh Stack op (`uvconvert.meshop.item`,
//! TYPE-IDS.md): the mesh becomes its UV layout in the map UV Name, as
//! Convert UVs to Mesh makes it (UVT-120): a flat mesh with U along X and
//! V along Y, a polygon per mapped polygon, keeping the map.
//!
//! Provisional (MOQ-55): the op's result is the layout alone (the command
//! makes a new item, which an op can't); with a Selection only the chosen
//! polygons are laid out. A map the mesh doesn't have passes it through.
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UvToMeshOp {
    pub uv_name: String,
}
impl Default for UvToMeshOp {
    fn default() -> Self {
        Self {
            uv_name: "Texture".into(),
        }
    }
}
impl TypedOp for UvToMeshOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if !ctx.input.uv_maps.contains_key(&self.uv_name) {
            return Ok(ctx.input.clone());
        }
        let source = match ctx.selection {
            None => ctx.input.clone(),
            Some(chosen) => {
                let on: BTreeSet<u32> = chosen.iter().copied().collect();
                let gone = (0..ctx.input.faces.len())
                    .filter(|f| !on.contains(&(*f as u32)))
                    .collect();
                crate::meshop_curve_common::drop_faces(ctx.input, &gone)?
            }
        };
        crate::uv_export::uvs_to_mesh(&source, &self.uv_name)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<UvToMeshOp>::new(
        "uvconvert.meshop.item",
        "Convert UV to Mesh",
        "UV",
        true,
        vec![Field::new("uvName", "UV Map", FieldKind::Text)],
    ))]
}
