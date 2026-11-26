//! UV Flip as a Mesh Stack op (`uvflip.meshop.item`, TYPE-IDS.md): the
//! chosen polygons' UVs in the map Name mirrored in U or V about their
//! bounding box's centre, as the Flip button does (UVT-51), all together
//! or (Flip per Polygon) each polygon on its own. A map the mesh doesn't
//! have passes the mesh through.
use crate::mesh_stack_builtin::{check, choice};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::uv_edit::{Grouping, IslandOp, IslandRule, UvAxis};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UvFlipOp {
    pub uv_name: String,
    pub sepa: bool,
    /// 0 U, 1 V.
    pub axis: u32,
}
impl Default for UvFlipOp {
    fn default() -> Self {
        Self {
            uv_name: "Texture".into(),
            sepa: false,
            axis: 0,
        }
    }
}
impl TypedOp for UvFlipOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let axis = match self.axis {
            0 => UvAxis::U,
            1 => UvAxis::V,
            _ => return Err("Axis must be 0 (U) or 1 (V)".into()),
        };
        let mut out = ctx.input.clone();
        if !out.uv_maps.contains_key(&self.uv_name) {
            return Ok(out);
        }
        let selection = crate::Selection {
            mode: crate::Mode::Polygon,
            faces: ctx.chosen()?,
            ..Default::default()
        };
        let plan = crate::uv_edit::plan(&out, &self.uv_name, &selection, &[], false)?;
        let op = IslandOp {
            rule: IslandRule::Flip { axis },
            grouping: if self.sepa {
                Grouping::Polygon
            } else {
                Grouping::Entire
            },
        };
        crate::uv_edit::apply_island_op(&mut out, &plan, &op)?;
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<UvFlipOp>::new(
        "uvflip.meshop.item",
        "UV Flip",
        "UV",
        true,
        vec![
            Field::new("uvName", "UV Map", FieldKind::Text),
            check("sepa", "Flip per Polygon"),
            choice("axis", "Axis", &["U", "V"]),
        ],
    ))]
}
