//! Helpers the vertex map ops share (Mesh Stack batch 4).
use crate::meshop_types::{OpContext, SelElement};

/// The points a point op works on: the chosen points, or every point.
/// An op given polygons works on their points.
pub(crate) fn chosen_points(ctx: &OpContext) -> Vec<u32> {
    let n = ctx.input.vertices.len() as u32;
    let Some(picked) = ctx.selection else {
        return (0..n).collect();
    };
    if ctx.element == SelElement::Point {
        return picked.iter().copied().filter(|&v| v < n).collect();
    }
    let mut out = std::collections::BTreeSet::new();
    for &f in picked {
        out.extend(ctx.input.faces[f as usize].vertices.iter().copied());
    }
    out.into_iter().collect()
}

/// The weight map a blank Map Name means: the mesh's first, or "Weight".
pub(crate) fn weight_map_name(mesh: &crate::Mesh, name: &str) -> String {
    let name = name.trim();
    if !name.is_empty() {
        return name.to_owned();
    }
    mesh.attributes
        .point_weights
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "Weight".into())
}
