//! Wrap as a Mesh Stack deformer (`deform.wrap.item`, Operandi's own id):
//! each point sticks to one spot on a Target mesh and follows it when the
//! target moves or changes shape.
//!
//! The binding (which spot each point sticks to, and how far off the
//! surface it sits) is saved with the step (Jason, 2026-11-26: option 1),
//! so the wrapped mesh keeps following the target after save and reopen.
//! Rebind ([`bind`]) measures it again from where things are now.
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own. A spot
//! is a polygon's triangle plus two weights; when the target's polygons
//! change so the saved spots no longer exist, the step passes the mesh
//! through and asks for a Rebind. Rebind is a Properties button (it
//! writes the Binding value with the ordinary set-values command).
use crate::meshop_deformer_common::{FOLDER, deform_points, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, OpNote, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const WRAP_ID: &str = "deform.wrap.item";

/// One point's binding: target polygon, the triangle's three corners (as
/// places in that polygon), the weights of the 2nd and 3rd corner, then
/// the point's offset along the triangle's edge, across it and along its
/// normal.
pub type Spot = [f64; 9];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WrapOp {
    /// The target mesh item (0 = none).
    pub target: u64,
    pub strength: f64,
    /// One spot per point of the step's input; empty until Rebind.
    pub binding: Vec<Spot>,
}
impl Default for WrapOp {
    fn default() -> Self {
        Self {
            target: 0,
            strength: 1.,
            binding: Vec::new(),
        }
    }
}

/// The point, edge direction, across direction and normal of a target
/// triangle, or `None` when it has no area.
fn frame(m: &Mesh, f: usize, c: [usize; 3]) -> Option<[DVec3; 4]> {
    let face = m.faces.get(f)?;
    let [a, b, c] = c.map(|i| {
        face.vertices
            .get(i)
            .and_then(|&v| m.vertices.get(v as usize))
    });
    let (a, b, c) = (*a?, *b?, *c?);
    let n = (b - a).cross(c - a).try_normalize()?;
    let t = (b - a).try_normalize()?;
    Some([a, t, n.cross(t), n])
}
fn corners(m: &Mesh, f: usize, c: [usize; 3]) -> Option<[DVec3; 3]> {
    let face = m.faces.get(f)?;
    let [a, b, c] = c.map(|i| {
        face.vertices
            .get(i)
            .and_then(|&v| m.vertices.get(v as usize))
    });
    Some([*a?, *b?, *c?])
}

/// Where a bound point goes on `target` as it is now, or `None` when the
/// spot no longer exists there.
fn place(target: &Mesh, s: &Spot) -> Option<DVec3> {
    if s.iter().any(|v| !v.is_finite()) || s[..4].iter().any(|&v| v < 0.) {
        return None;
    }
    let (f, c) = (s[0] as usize, [s[1] as usize, s[2] as usize, s[3] as usize]);
    let [a, b, cc] = corners(target, f, c)?;
    let [_, t, across, n] = frame(target, f, c)?;
    let on = a * (1. - s[4] - s[5]) + b * s[4] + cc * s[5];
    Some(on + t * s[6] + across * s[7] + n * s[8])
}

fn in_local(mesh: &Mesh, m: DMat4) -> Mesh {
    let mut out = mesh.clone();
    out.vertices
        .iter_mut()
        .for_each(|p| *p = m.transform_point3(*p));
    out
}

/// The binding of `points` to `target` (both in the same space): each
/// point's nearest spot and its offset from it.
pub fn binding_of(points: &[DVec3], target: &Mesh) -> Result<Vec<Spot>> {
    let mut tris = Vec::new();
    for f in 0..target.faces.len() {
        for t in target.triangles(f)? {
            tris.push((f, t));
        }
    }
    if tris.is_empty() {
        return Err("Wrap: the target has no polygons".into());
    }
    points
        .iter()
        .map(|&p| {
            let (f, c, w) = crate::generator_ops::nearest_on(target, &tris, p)
                .ok_or("Wrap: the target has no polygons")?;
            let Some([a, t, across, n]) = frame(target, f, c) else {
                // A triangle with no area has no frame: stick without an
                // offset to the nearest corner's spot.
                return Ok([
                    f as f64,
                    c[0] as f64,
                    c[1] as f64,
                    c[2] as f64,
                    w.y,
                    w.z,
                    0.,
                    0.,
                    0.,
                ]);
            };
            let [_, b, cc] = corners(target, f, c).unwrap_or([a; 3]);
            let d = p - (a * w.x + b * w.y + cc * w.z);
            Ok([
                f as f64,
                c[0] as f64,
                c[1] as f64,
                c[2] as f64,
                w.y,
                w.z,
                d.dot(t),
                d.dot(across),
                d.dot(n),
            ])
        })
        .collect()
}

/// Rebind: the Binding value for Wrap step `op` of `item`, measured from
/// the step's input and its target as they are now (evaluated afresh, so
/// it never reads a stale result).
pub fn bind(scene: &crate::Scene, item: u64, op: u64) -> Result<serde_json::Value> {
    let it = scene.item(item).ok_or("Wrap: the item is gone")?;
    let stack = it
        .mesh_stack
        .as_ref()
        .ok_or("Wrap: the item has no Mesh Stack")?;
    let index = stack.index_of(op).ok_or("Wrap: the step is gone")?;
    let values: WrapOp =
        serde_json::from_value(stack.ops[index].kind.values()).map_err(|e| format!("Wrap: {e}"))?;
    if values.target == 0 {
        return Err("Wrap: pick a Target first".into());
    }
    let runtime = crate::mesh_stack::Runtime::default();
    // The step's input: the stack evaluated up to (not including) it.
    let mut below = scene.clone();
    let input: Mesh = {
        let cut = below.item_mut(item).ok_or("Wrap: the item is gone")?;
        if let Some(s) = cut.mesh_stack.as_mut() {
            s.ops.truncate(index);
        }
        let cut = below.item(item).ok_or("Wrap: the item is gone")?;
        match runtime.current(&below, cut) {
            Some(e) => (*e.mesh).clone(),
            None => (**cut.mesh.shared()).clone(),
        }
    };
    let target = scene
        .item(values.target)
        .filter(|t| t.is_mesh() && t.id != item)
        .ok_or("Wrap: the Target isn't a mesh item")?;
    let target_mesh: Mesh = match runtime.current(scene, target) {
        Some(e) => (*e.mesh).clone(),
        None => (**target.mesh.shared()).clone(),
    };
    let to_local = scene.world_to_local(item)? * scene.world_matrix(values.target)?;
    let spots = binding_of(&input.vertices, &in_local(&target_mesh, to_local))?;
    serde_json::to_value(spots).map_err(|e| e.to_string())
}

impl TypedOp for WrapOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let Some(source) = ctx.sources()?.into_iter().next() else {
            return Ok(ctx.input.clone());
        };
        let pass = |text: &str| {
            ctx.note(OpNote {
                item: 0,
                text: text.into(),
                problem: true,
            });
            Ok(ctx.input.clone())
        };
        if self.binding.is_empty() {
            return pass("Not bound yet: press Rebind to stick the points to the Target");
        }
        if self.binding.len() != ctx.input.vertices.len() {
            return pass("The mesh below changed its points since binding: press Rebind");
        }
        let target = in_local(&source.mesh, source.to_local);
        let places: Vec<Option<DVec3>> = self.binding.iter().map(|s| place(&target, s)).collect();
        if places.iter().any(Option::is_none) {
            return pass("The Target's polygons changed since binding: press Rebind");
        }
        deform_points(ctx, self.strength, |i, p| places[i].unwrap_or(p))
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<WrapOp>::new(
            WRAP_ID,
            "Wrap",
            FOLDER,
            true,
            vec![
                Field::new("target", "Target", FieldKind::Item),
                strength(),
                Field::new("binding", "Binding", FieldKind::Hidden),
            ],
        ))],
        SelElement::Point,
    )
}
