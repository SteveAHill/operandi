//! What every deformer step in the Mesh Stack shares (deformers plan
//! 2026-11-26): a deformer only moves points, never adds or removes them,
//! so it runs in the chain like any step and the cache re-runs it only
//! when something below it (or its own values) changes. Each one moves
//! the points its Selection picked (all with none) by its Strength; a
//! Falloff step below shapes it further (batch 7).
//!
//! Provisional (MOQ-68): deformer values and defaults are Operandi's own
//! (the reference's deformer channels aren't in TYPE-IDS); they work in
//! the mesh's own space.
use crate::meshop_types::{Field, FieldKind, OpContext, SelElement};
use crate::{Mesh, Result};
use glam::DVec3;

/// The Add browser folder deformers live in (its own root).
pub(crate) const FOLDER: &str = "Deformers";

/// The Strength row every deformer has.
pub(crate) fn strength() -> Field {
    Field::new("strength", "Strength", FieldKind::Percent)
}

/// Moves the chosen points of the op's input to `to(p)`, by `strength`
/// (0..1) of the way.
pub(crate) fn deform(ctx: &OpContext, strength: f64, to: impl Fn(DVec3) -> DVec3) -> Result<Mesh> {
    deform_points(ctx, strength, |_, p| to(p))
}

/// [`deform`] where the new place also depends on the point's number.
pub(crate) fn deform_points(
    ctx: &OpContext,
    strength: f64,
    to: impl Fn(usize, DVec3) -> DVec3,
) -> Result<Mesh> {
    if !strength.is_finite() {
        return Err("Strength must be a number".into());
    }
    let mut out = ctx.input.clone();
    let mut moved = |i: usize| {
        if let Some(p) = out.vertices.get_mut(i) {
            let q = to(i, *p);
            if q.is_finite() {
                *p += (q - *p) * strength;
            }
        }
    };
    match (ctx.selection, ctx.element) {
        (None, _) => (0..ctx.input.vertices.len()).for_each(&mut moved),
        (Some(points), SelElement::Point) => points.iter().for_each(|&i| moved(i as usize)),
        (Some(chosen), element) => {
            let mut points = std::collections::BTreeSet::new();
            for &c in chosen {
                match element {
                    SelElement::Polygon => {
                        if let Some(f) = ctx.input.faces.get(c as usize) {
                            points.extend(f.vertices.iter().copied());
                        }
                    }
                    _ => {
                        if let Some(&(a, b)) = ctx.input.edges().get(c as usize) {
                            points.extend([a, b]);
                        }
                    }
                }
            }
            points.into_iter().for_each(|i| moved(i as usize));
        }
    }
    Ok(out)
}

/// A deformer's Axis choice (0 X, 1 Y, 2 Z) as a direction.
pub(crate) fn axis_of(axis: u32, what: &str) -> Result<DVec3> {
    match axis {
        0 => Ok(DVec3::X),
        1 => Ok(DVec3::Y),
        2 => Ok(DVec3::Z),
        _ => Err(format!("{what} Axis must be 0 (X), 1 (Y) or 2 (Z)")),
    }
}

/// A Map Name row's weight map on `mesh`; an empty name means every
/// point weighs 1.
pub(crate) fn weights<'m>(
    mesh: &'m Mesh,
    name: &str,
    what: &str,
) -> Result<Option<&'m std::collections::BTreeMap<u32, f64>>> {
    if name.trim().is_empty() {
        return Ok(None);
    }
    mesh.attributes
        .point_weights
        .get(name)
        .map(Some)
        .ok_or_else(|| format!("{what}: the mesh has no weight map \u{201c}{name}\u{201d}"))
}
