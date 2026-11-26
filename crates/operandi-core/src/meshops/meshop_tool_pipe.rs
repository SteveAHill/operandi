//! Tool pipe steps in the Mesh Stack (batch 6): Symmetry
//! (`symmetry.tool.item`) and Set Work Plane (`pmodel.workplane.item`),
//! TYPE-IDS.md. Neither changes the mesh itself; each sets something the
//! steps above it use, until another step of the same kind replaces it.
//!
//! - **Symmetry**: a step above that has a Selection also works on the
//!   mirror image of the elements it picked, across the mesh's own
//!   X, Y or Z = 0 plane (elements with no mirror partner are left as
//!   they are).
//! - **Set Work Plane**: the primitives and curve shapes above (the Create
//!   steps) are made on this plane: moved by Offset and turned by Angle X,
//!   then Y, then Z, about the mesh's origin.
//!
//! Provisional (MOQ-66): which steps follow each setting, the mirror's
//! plane and tolerance, and the angle order are Operandi's reading; the
//! reference's tool pipe entries also drive interactive tools, which the
//! stack doesn't have. A turned-off step sets nothing.
use crate::mesh_stack::{MeshOp, OpKind};
use crate::mesh_stack_builtin::{choice, distance};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DMat4, DQuat, DVec3, EulerRot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub const SYMMETRY_ID: &str = "symmetry.tool.item";
pub const WORK_PLANE_ID: &str = "pmodel.workplane.item";

/// The steps Set Work Plane places: those that make new geometry from
/// their own values.
const PLACED: &[&str] = &[
    "curve.circle.item",
    "curve.diamond.item",
    "curve.ellipse.item",
    "curve.nsided.item",
    "curve.rectangle.item",
    "curve.star.item",
    "prim.arc.item",
    "prim.capsule.item",
    "prim.cone.item",
    "prim.cube.item",
    "prim.cylinder.item",
    "prim.ellipsoid.item",
    "prim.sphere.item",
    "prim.teapot.item",
    "prim.text.item",
    "prim.toroid.item",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymmetryOp {
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
}
impl TypedOp for SymmetryOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.axis > 2 {
            return Err("Symmetry Axis must be 0 (X), 1 (Y) or 2 (Z)".into());
        }
        Ok(ctx.input.clone())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkPlaneOp {
    pub offsetx: f64,
    pub offsety: f64,
    pub offsetz: f64,
    /// Radians.
    pub anglex: f64,
    pub angley: f64,
    pub anglez: f64,
}
impl WorkPlaneOp {
    /// The plane's frame: X turn first, then Y, then Z, then the offset.
    pub fn matrix(&self) -> Option<DMat4> {
        let v = [
            self.offsetx,
            self.offsety,
            self.offsetz,
            self.anglex,
            self.angley,
            self.anglez,
        ];
        v.iter().all(|x| x.is_finite()).then(|| {
            DMat4::from_rotation_translation(
                DQuat::from_euler(EulerRot::ZYX, self.anglez, self.angley, self.anglex),
                DVec3::new(self.offsetx, self.offsety, self.offsetz),
            )
        })
    }
}
impl TypedOp for WorkPlaneOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        self.matrix()
            .ok_or("Set Work Plane values must be numbers")?;
        Ok(ctx.input.clone())
    }
}

/// What the tool pipe steps below an op have set.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pipe {
    /// Mirror axis (0 X, 1 Y, 2 Z).
    pub symmetry: Option<usize>,
    pub work_plane: Option<DMat4>,
}
impl Pipe {
    /// The pipe for the ops above `op`.
    pub fn after(&mut self, op: &MeshOp) {
        if !op.enabled {
            return;
        }
        let OpKind::Op { type_id, values } = &op.kind else {
            return;
        };
        match type_id.as_str() {
            SYMMETRY_ID => {
                if let Ok(s) = serde_json::from_value::<SymmetryOp>(values.clone())
                    && s.axis <= 2
                {
                    self.symmetry = Some(s.axis as usize);
                }
            }
            WORK_PLANE_ID => {
                if let Some(m) = serde_json::from_value::<WorkPlaneOp>(values.clone())
                    .ok()
                    .and_then(|w| w.matrix())
                {
                    self.work_plane = Some(m);
                }
            }
            _ => {}
        }
    }

    /// `chosen` plus their mirror partners (Symmetry).
    pub fn mirrored(&self, mesh: &Mesh, element: SelElement, chosen: Vec<u32>) -> Vec<u32> {
        let Some(axis) = self.symmetry else {
            return chosen;
        };
        let partner = mirror_partners(mesh, element, axis);
        let mut out = chosen.clone();
        out.extend(chosen.iter().filter_map(|i| partner.get(i).copied()));
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Set Work Plane: moves the points `type_id` added (those past
    /// `before` points) onto the plane.
    pub fn place(&self, type_id: &str, before: usize, out: &mut Mesh) {
        let Some(m) = self.work_plane else {
            return;
        };
        if !PLACED.contains(&type_id) || m == DMat4::IDENTITY {
            return;
        }
        for p in out.vertices.iter_mut().skip(before) {
            *p = m.transform_point3(*p);
        }
    }
}

/// A key for a position, rounded to the mirror's tolerance.
fn key(p: DVec3, tolerance: f64) -> (i64, i64, i64) {
    let q = p / tolerance;
    (q.x.round() as i64, q.y.round() as i64, q.z.round() as i64)
}

/// Each element's partner across the axis = 0 plane, numbered as
/// `element` says.
fn mirror_partners(mesh: &Mesh, element: SelElement, axis: usize) -> HashMap<u32, u32> {
    let size = mesh
        .vertices
        .iter()
        .fold(0f64, |m, p| m.max(p.abs().max_element()));
    let tolerance = 1e-6 * size.max(1.);
    let flip = |mut p: DVec3| {
        p[axis] = -p[axis];
        p
    };
    // Points by rounded position (snapping both neighbours of a rounding
    // boundary isn't needed at this tolerance for meshes made symmetric).
    let at: HashMap<_, u32> = mesh
        .vertices
        .iter()
        .enumerate()
        .map(|(i, &p)| (key(p, tolerance), i as u32))
        .collect();
    let point_partner = |i: usize| at.get(&key(flip(mesh.vertices[i]), tolerance)).copied();
    match element {
        SelElement::Point => (0..mesh.vertices.len())
            .filter_map(|i| point_partner(i).map(|j| (i as u32, j)))
            .collect(),
        SelElement::Edge => {
            let edges = mesh.edges();
            let index: HashMap<(u32, u32), u32> = edges
                .iter()
                .enumerate()
                .map(|(i, &e)| (e, i as u32))
                .collect();
            edges
                .iter()
                .enumerate()
                .filter_map(|(i, &(a, b))| {
                    let (x, y) = (point_partner(a as usize)?, point_partner(b as usize)?);
                    let j = *index.get(&crate::topology::edge_key(x, y))?;
                    Some((i as u32, j))
                })
                .collect()
        }
        SelElement::Polygon => {
            // A polygon's partner uses the mirrored points.
            let mut by_points: HashMap<Vec<u32>, u32> = HashMap::new();
            for (f, face) in mesh.faces.iter().enumerate() {
                let mut v = face.vertices.clone();
                v.sort_unstable();
                by_points.insert(v, f as u32);
            }
            (0..mesh.faces.len())
                .filter_map(|f| {
                    let mut v: Vec<u32> = mesh.faces[f]
                        .vertices
                        .iter()
                        .map(|&p| point_partner(p as usize))
                        .collect::<Option<_>>()?;
                    v.sort_unstable();
                    Some((f as u32, *by_points.get(&v)?))
                })
                .collect()
        }
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let angle = |key, label| Field::new(key, label, FieldKind::Angle);
    vec![
        Arc::new(Typed::<SymmetryOp>::new(
            SYMMETRY_ID,
            "Symmetry",
            "Other",
            false,
            vec![choice("axis", "Axis", &["X", "Y", "Z"])],
        )),
        Arc::new(Typed::<WorkPlaneOp>::new(
            WORK_PLANE_ID,
            "Set Work Plane",
            "Other",
            false,
            vec![
                distance("offsetx", "Offset X"),
                distance("offsety", "Offset Y"),
                distance("offsetz", "Offset Z"),
                angle("anglex", "Angle X"),
                angle("angley", "Angle Y"),
                angle("anglez", "Angle Z"),
            ],
        )),
    ]
}
