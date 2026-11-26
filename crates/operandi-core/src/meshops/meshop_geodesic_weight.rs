//! Geodesic Weight as a Mesh Stack op (`pmodel.geodesic.mOp`,
//! TYPE-IDS.md): a weight map (Map Name) of each point's distance from
//! the chosen points, measured along the mesh's edges. Normalize scales
//! it so the farthest point is 100%; Invert turns it round so the chosen
//! points are 100% and the farthest 0%. Points the chosen ones can't
//! reach get no weight.
//!
//! Provisional (MOQ-60): distance runs along edges (shortest edge path),
//! not across polygons, so Triangulate and the advanced solver are kept,
//! not shown; the reference's runtime Input Array isn't saved, so the op's
//! Selection gives the start points. With no Selection the op adds nothing.
use crate::mesh_stack_builtin::check;
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::meshop_vertex_map_common::chosen_points;
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeodesicWeightOp {
    pub normalize: bool,
    pub advanced: bool,
    pub triangulate: bool,
    pub invert: bool,
    pub mapname: String,
}
impl Default for GeodesicWeightOp {
    fn default() -> Self {
        Self {
            normalize: true,
            advanced: true,
            triangulate: true,
            invert: false,
            mapname: "Geodesic".into(),
        }
    }
}

/// Shortest edge-path distance from `sources` to every point it reaches.
pub(crate) fn edge_distances(mesh: &Mesh, sources: &[u32]) -> Vec<Option<f64>> {
    let n = mesh.vertices.len();
    let mut next: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (a, b) in mesh.edges() {
        next[a as usize].push(b);
        next[b as usize].push(a);
    }
    let mut best: Vec<Option<f64>> = vec![None; n];
    // Distances as ordered bits: non-negative floats sort like their bits.
    let mut queue = BinaryHeap::new();
    for &s in sources {
        best[s as usize] = Some(0.);
        queue.push(Reverse((0u64, s)));
    }
    while let Some(Reverse((bits, v))) = queue.pop() {
        let d = f64::from_bits(bits);
        if best[v as usize].is_some_and(|b| b < d) {
            continue;
        }
        for &w in &next[v as usize] {
            let nd = d + mesh.vertices[v as usize].distance(mesh.vertices[w as usize]);
            if best[w as usize].is_none_or(|b| nd < b) {
                best[w as usize] = Some(nd);
                queue.push(Reverse((nd.to_bits(), w)));
            }
        }
    }
    best
}

impl TypedOp for GeodesicWeightOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let mut out = ctx.input.clone();
        if ctx.selection.is_none() {
            return Ok(out);
        }
        let sources = chosen_points(ctx);
        let found = edge_distances(&out, &sources);
        let far = found.iter().flatten().fold(0f64, |m, &d| m.max(d));
        let mut weights = BTreeMap::new();
        for (v, d) in found.iter().enumerate() {
            let Some(d) = *d else {
                continue;
            };
            let mut w = if self.normalize && far > 0. {
                d / far
            } else {
                d
            };
            if self.invert {
                w = if self.normalize { 1. - w } else { far - w };
            }
            weights.insert(v as u32, w);
        }
        let name = match self.mapname.trim() {
            "" => "Geodesic".to_owned(),
            n => n.to_owned(),
        };
        out.attributes.point_weights.insert(name, weights);
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<GeodesicWeightOp>::new(
            "pmodel.geodesic.mOp",
            "Geodesic Weight",
            "Vertex Map",
            true,
            vec![
                Field::new("mapname", "Map Name", FieldKind::Text),
                check("normalize", "Normalize"),
                check("invert", "Invert"),
                hidden("advanced"),
                hidden("triangulate"),
            ],
        ))],
        SelElement::Point,
    )
}
