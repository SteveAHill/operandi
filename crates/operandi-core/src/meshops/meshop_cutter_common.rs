//! The Slice cutter's values (`effector.cutter.item`, TYPE-IDS.md), shared
//! by the path slice ops of Mesh Stack batch 5: Curve Slice, Pen Slice and
//! Primitive Slice. Each op makes its paths and cuts them in with the
//! direct tools' engine (`path_slice::path_cut`, PS-10..17), in the mesh's
//! own space.
//!
//! Provisional (MOQ-63): Operation lists only the built cuts (Core, Tunnel,
//! Stencil, Slice); the boolean types' Negative/Positive Depth and preview
//! are kept but not shown. Select Sliced Edges adds the cut's new edges to
//! the edge selection set named Edge Selection Set.
use crate::mesh_stack_builtin::{check, choice, distance};
use crate::meshop_types::{Field, FieldKind, OpContext};
use crate::path_slice::{CutPath, PathCutter, SliceAxis, SliceOperation, path_cut};
use crate::{Mesh, Result};
use glam::{DMat4, DVec3};

/// The cutter's Axis choices, by position.
pub(crate) const AXES: [&str; 5] = ["X", "Y", "Z", "Custom", "Action Axis"];
/// Position of Action Axis in [`AXES`].
pub(crate) const ACTION_AXIS: u32 = 4;
/// The Operation choices that are built, by position.
pub(crate) const OPERATIONS: [&str; 4] = ["Core", "Tunnel", "Stencil", "Slice"];

/// An op's value struct with the cutter's channels after its own. Every
/// op keeps `deny_unknown_fields`, so the cutter's values are written out
/// in each struct rather than flattened.
macro_rules! cutter_op {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $( $(#[$fmeta:meta])* $field:ident : $ty:ty = $default:expr, )*
        }
        axis = $axis:expr;
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        pub struct $name {
            $( $(#[$fmeta])* pub $field: $ty, )*
            /// The cutter's Axis, by position in `AXES`.
            pub axis: u32,
            pub vector_x: f64,
            pub vector_y: f64,
            pub vector_z: f64,
            pub select_edges: bool,
            pub selection_set_name: String,
            /// Operation, by position in `OPERATIONS`.
            pub mode: u32,
            pub material: String,
            pub front_only: bool,
            /// Boolean types only (not built): kept, not shown.
            pub depth0: f64,
            pub depth1: f64,
            pub preview: bool,
        }
        impl Default for $name {
            fn default() -> Self {
                Self {
                    $( $field: $default, )*
                    axis: $axis,
                    vector_x: 0.,
                    vector_y: 1.,
                    vector_z: 0.,
                    select_edges: false,
                    selection_set_name: "Sliced Edges".into(),
                    mode: 3,
                    material: "Default".into(),
                    front_only: false,
                    depth0: 0.,
                    depth1: 1.,
                    preview: true,
                }
            }
        }
        impl $name {
            /// The cutter these values describe.
            fn cutter(&self) -> $crate::Result<$crate::meshop_cutter_common::Cutter> {
                $crate::meshop_cutter_common::Cutter::new(
                    self.axis,
                    [self.vector_x, self.vector_y, self.vector_z],
                    self.mode,
                    &self.material,
                    self.front_only,
                    self.select_edges,
                    &self.selection_set_name,
                )
            }
        }
    };
}
pub(crate) use cutter_op;

/// The cutter, checked, plus where its sliced edges go.
pub(crate) struct Cutter {
    pub cutter: PathCutter,
    /// `Some(set name)` when Select Sliced Edges is on.
    pub edge_set: Option<String>,
}
impl Cutter {
    pub(crate) fn new(
        axis: u32,
        vector: [f64; 3],
        mode: u32,
        material: &str,
        front_only: bool,
        select_edges: bool,
        set_name: &str,
    ) -> Result<Self> {
        crate::mesh_stack_builtin::finite(&vector, "Vector")?;
        let axis = *SliceAxis::ALL
            .get(axis as usize)
            .ok_or("Axis must be 0 (X) to 4 (Action Axis)")?;
        let operation = match mode {
            0 => SliceOperation::Core,
            1 => SliceOperation::Tunnel,
            2 => SliceOperation::Stencil,
            3 => SliceOperation::Slice,
            _ => return Err("Operation must be 0 (Core) to 3 (Slice)".into()),
        };
        let set_name = set_name.trim();
        if select_edges && set_name.is_empty() {
            return Err("Select Sliced Edges needs an Edge Selection Set name".into());
        }
        Ok(Self {
            cutter: PathCutter {
                operation,
                axis,
                vector: DVec3::from_array(vector),
                front_face_only: front_only,
                material: material.into(),
                select_sliced_edges: select_edges,
                edge_set: set_name.into(),
                ..PathCutter::default()
            },
            edge_set: select_edges.then(|| set_name.to_string()),
        })
    }

    /// Cuts `paths` into the op's input along the cutter's direction
    /// (`action` is the op's own Action Axis). The chosen polygons are cut,
    /// or every polygon with no Selection; curves are never cut.
    pub(crate) fn cut(&self, ctx: &OpContext, paths: &[CutPath], action: DVec3) -> Result<Mesh> {
        if paths.iter().all(|p| p.points.len() < 2) {
            return Ok(ctx.input.clone());
        }
        let direction = self
            .cutter
            .direction(action)
            .ok_or("The slice axis has no direction (Custom needs a Vector)")?;
        let r = path_cut(
            ctx.input,
            DMat4::IDENTITY,
            paths,
            direction,
            &self.cutter,
            &ctx.chosen()?,
        )?;
        let mut mesh = r.mesh;
        if let Some(name) = &self.edge_set {
            mesh.attributes
                .edge_sets
                .entry(name.clone())
                .or_default()
                .extend(r.new_edges);
        }
        Ok(mesh)
    }
}

/// The cutter's fields, after the op's own.
pub(crate) fn cutter_fields() -> Vec<Field> {
    let hidden = |key: &'static str| Field::new(key, key, FieldKind::Hidden);
    vec![
        choice("axis", "Axis", &AXES),
        distance("vectorX", "Vector X"),
        distance("vectorY", "Vector Y"),
        distance("vectorZ", "Vector Z"),
        choice("mode", "Operation", &OPERATIONS),
        Field::new("material", "Material", FieldKind::Text),
        check("frontOnly", "Front Face Only"),
        check("selectEdges", "Select Sliced Edges"),
        Field::new("selectionSetName", "Edge Selection Set", FieldKind::Text),
        hidden("depth0"),
        hidden("depth1"),
        hidden("preview"),
    ]
}
