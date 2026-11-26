//! Set Vertex Normals as a Mesh Stack op (`setvmap.vnorm.item`,
//! TYPE-IDS.md): the vertex normal map Map Name rebuilt, as Recalculate
//! Normals does (VN-1/2, the Options' weighting, splitting at hard edges),
//! on the chosen points' corners; other corners keep what they had.
//!
//! Provisional (MOQ-62): Tolerance, Method, Angle, Power and Color Map
//! Name are kept, not shown or built (their meaning isn't specced); the
//! normals use the Vertex Normal Options' defaults.
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::meshop_vertex_map_common::chosen_points;
use crate::vertex_normals::{DEFAULT_MAP, NormalOptions, recalculate};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SetVertexNormalsOp {
    #[serde(rename = "MapName")]
    pub map_name: String,
    pub tolerance: f64,
    pub method: u32,
    pub angle: f64,
    pub power: u32,
    pub color_map_name: String,
}
impl Default for SetVertexNormalsOp {
    fn default() -> Self {
        Self {
            map_name: DEFAULT_MAP.into(),
            tolerance: 0.1,
            method: 0,
            angle: 0.2,
            power: 0,
            color_map_name: String::new(),
        }
    }
}
impl TypedOp for SetVertexNormalsOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let name = match self.map_name.trim() {
            "" => DEFAULT_MAP.to_owned(),
            n => n.to_owned(),
        };
        let o = NormalOptions {
            use_custom_name: true,
            map_name: name.clone(),
            ..Default::default()
        };
        let mut out = recalculate(ctx.input, &o)?;
        if ctx.selection.is_some() {
            let on: std::collections::BTreeSet<u32> = chosen_points(ctx).into_iter().collect();
            let before = ctx.input.attributes.corner_normals.get(&name).cloned();
            let faces = &ctx.input.faces;
            if let Some(rows) = out.attributes.corner_normals.get_mut(&name) {
                for (f, row) in rows.iter_mut().enumerate() {
                    for (c, n) in row.iter_mut().enumerate() {
                        if !on.contains(&faces[f].vertices[c]) {
                            *n = before
                                .as_ref()
                                .and_then(|b| b.get(f)?.get(c).copied().flatten());
                        }
                    }
                }
            }
        }
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<SetVertexNormalsOp>::new(
            "setvmap.vnorm.item",
            "Set Vertex Normals",
            "Vertex Map",
            true,
            vec![
                Field::new("MapName", "Map Name", FieldKind::Text),
                hidden("tolerance"),
                hidden("method"),
                hidden("angle"),
                hidden("power"),
                hidden("colorMapName"),
            ],
        ))],
        SelElement::Point,
    )
}
