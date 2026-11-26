//! Lattice as a Mesh Stack deformer (`deform.lattice.item`, Operandi's own
//! id): a box cage of Divisions X x Y x Z control points; moving a
//! control point (its handle in the view while the step is selected)
//! bends the points inside the box smoothly with it (a Bernstein free-form
//! deformation: the cage as it was moves nothing).
//!
//! Provisional (MOQ-68): values and defaults are Operandi's own; the cage
//! is the box Min..Max in the mesh's space (default the 1 m cube around
//! the origin); points outside it don't move. Control point offsets are
//! kept per point, X fastest, then Y, then Z; changing Divisions starts
//! them over.
use crate::mesh_stack_builtin::{count, distance};
use crate::meshop_deformer_common::{FOLDER, deform, strength};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const LATTICE_ID: &str = "deform.lattice.item";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LatticeOp {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
    pub div_x: u32,
    pub div_y: u32,
    pub div_z: u32,
    /// Each control point's move from its place in the box; empty or the
    /// wrong length = none moved.
    pub offsets: Vec<[f64; 3]>,
    pub strength: f64,
}
impl Default for LatticeOp {
    fn default() -> Self {
        Self {
            min_x: -0.5,
            min_y: -0.5,
            min_z: -0.5,
            max_x: 0.5,
            max_y: 0.5,
            max_z: 0.5,
            div_x: 3,
            div_y: 3,
            div_z: 3,
            offsets: Vec::new(),
            strength: 1.,
        }
    }
}

/// Bernstein basis value `i` of degree `n` at `t`.
fn bernstein(n: usize, i: usize, t: f64) -> f64 {
    let mut c = 1f64;
    for k in 0..i {
        c = c * (n - k) as f64 / (k + 1) as f64;
    }
    c * t.powi(i as i32) * (1. - t).powi((n - i) as i32)
}

impl LatticeOp {
    pub fn divisions(&self) -> Result<[usize; 3]> {
        let d = [self.div_x, self.div_y, self.div_z];
        if d.iter().any(|&n| !(2..=16).contains(&n)) {
            return Err("Lattice Divisions must be 2 to 16".into());
        }
        Ok(d.map(|n| n as usize))
    }
    pub fn bounds(&self) -> Result<(DVec3, DVec3)> {
        let lo = DVec3::new(self.min_x, self.min_y, self.min_z);
        let hi = DVec3::new(self.max_x, self.max_y, self.max_z);
        if !(lo.is_finite() && hi.is_finite()) || (hi - lo).min_element() <= 1e-9 {
            return Err("Lattice Max must be past Min on every axis".into());
        }
        Ok((lo, hi))
    }
    /// The control points' offsets, one per point (zero when unset).
    pub fn moves(&self) -> Result<Vec<DVec3>> {
        let [a, b, c] = self.divisions()?;
        let n = a * b * c;
        if self.offsets.len() != n {
            return Ok(vec![DVec3::ZERO; n]);
        }
        let out: Vec<DVec3> = self.offsets.iter().map(|&o| DVec3::from(o)).collect();
        if !out.iter().all(|o| o.is_finite()) {
            return Err("Lattice offsets must be numbers".into());
        }
        Ok(out)
    }
    /// Control point `(i, j, k)`'s number.
    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.div_x as usize * (j + self.div_y as usize * k)
    }
    /// Every control point where it is now, in index order.
    pub fn control_points(&self) -> Result<Vec<DVec3>> {
        let [a, b, c] = self.divisions()?;
        let (lo, hi) = self.bounds()?;
        let moves = self.moves()?;
        let mut out = Vec::with_capacity(a * b * c);
        for k in 0..c {
            for j in 0..b {
                for i in 0..a {
                    let t = DVec3::new(
                        i as f64 / (a - 1) as f64,
                        j as f64 / (b - 1) as f64,
                        k as f64 / (c - 1) as f64,
                    );
                    out.push(lo + (hi - lo) * t + moves[self.index(i, j, k)]);
                }
            }
        }
        Ok(out)
    }
}

impl TypedOp for LatticeOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let [a, b, c] = self.divisions()?;
        let (lo, hi) = self.bounds()?;
        let moves = self.moves()?;
        if moves.iter().all(|m| *m == DVec3::ZERO) {
            return Ok(ctx.input.clone());
        }
        deform(ctx, self.strength, |p| {
            let t = (p - lo) / (hi - lo);
            if t.min_element() < 0. || t.max_element() > 1. {
                return p;
            }
            let mut moved = DVec3::ZERO;
            for k in 0..c {
                let wk = bernstein(c - 1, k, t.z);
                for j in 0..b {
                    let wj = wk * bernstein(b - 1, j, t.y);
                    for i in 0..a {
                        let m = moves[self.index(i, j, k)];
                        if m != DVec3::ZERO {
                            moved += m * wj * bernstein(a - 1, i, t.x);
                        }
                    }
                }
            }
            p + moved
        })
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<LatticeOp>::new(
            LATTICE_ID,
            "Lattice",
            FOLDER,
            true,
            vec![
                distance("minX", "Min X"),
                distance("minY", "Min Y"),
                distance("minZ", "Min Z"),
                distance("maxX", "Max X"),
                distance("maxY", "Max Y"),
                distance("maxZ", "Max Z"),
                count("divX", "Divisions X", 2, 16),
                count("divY", "Divisions Y", 2, 16),
                count("divZ", "Divisions Z", 2, 16),
                // Set by dragging the control points, not typed.
                Field::new("offsets", "Offsets", FieldKind::Hidden),
                strength(),
            ],
        ))],
        SelElement::Point,
    )
}
