//! Set Seam as a Mesh Stack op (`setvmap.seam.item`, TYPE-IDS.md): the
//! chosen edges join the UV seam map Map Name (made when missing), as Add
//! Seam does (UVS-1 §1.6), so a UV Unwrap above can cut there. With no
//! Selection every edge joins, as Add Seam with nothing selected.
//!
//! Provisional (MOQ-61): a blank Map Name means Add Seam's first map,
//! "UV Seam" (UVS-P02).
use crate::mesh_stack_tools::chosen_edges;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::uv_seams::{DEFAULT_SEAM_MAP, SEAM_PREFIX};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetSeamOp {
    #[serde(rename = "MapName")]
    pub map_name: String,
}
impl TypedOp for SetSeamOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let name = match self.map_name.trim() {
            "" => DEFAULT_SEAM_MAP,
            n => n,
        };
        let mut out = ctx.input.clone();
        let set = out
            .attributes
            .edge_sets
            .entry(format!("{SEAM_PREFIX}{name}"))
            .or_default();
        set.extend(
            chosen_edges(ctx)
                .into_iter()
                .map(|(a, b)| crate::topology::edge_key(a, b)),
        );
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<SetSeamOp>::new(
            "setvmap.seam.item",
            "Set Seam",
            "Vertex Map",
            true,
            vec![Field::new("MapName", "Map Name", FieldKind::Text)],
        ))],
        SelElement::Edge,
    )
}
