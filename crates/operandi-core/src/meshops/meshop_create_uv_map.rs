//! Create UV Map as a Mesh Stack op (`uv.create.item`, TYPE-IDS.md): the
//! UV Project tool's projections (`uv_project::project_mesh`, UVT §2) of
//! the chosen polygons into the map Name: Planar, Cylindrical, Spherical,
//! Atlas, Barycentric, Atlas 2 or Cubic, sized to the polygons (Settings
//! Automatic) or by Center, Size and Rotation (Manual).
//!
//! Provisional (MOQ-52): Projection Type is stored by position in the
//! tool's list; World Space is kept, not shown (the op works in the mesh's
//! own space); Create New Map off writes into the mesh's current map when
//! Name doesn't exist.
use crate::mesh_stack_builtin::{check, choice, count, distance};
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::uv_project::{ProjectOptions, ProjectionKind};
use crate::uv_tools::Axis3;
use crate::{Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CreateUvMapOp {
    /// Position in `ProjectionKind::ALL`.
    pub proj: u32,
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
    /// 0 Automatic, 1 Manual.
    pub mode: u32,
    pub cen_x: f64,
    pub cen_y: f64,
    pub cen_z: f64,
    pub siz_x: f64,
    pub siz_y: f64,
    pub siz_z: f64,
    /// Radians.
    pub rot_h: f64,
    pub rot_p: f64,
    pub rot_b: f64,
    pub gaps: f64,
    pub seg_mat: bool,
    pub part: bool,
    pub orient: bool,
    /// Radians.
    pub seam: f64,
    pub nseg: u32,
    pub iter: u32,
    pub world: bool,
    pub name: String,
    pub newmap: bool,
}
impl Default for CreateUvMapOp {
    fn default() -> Self {
        Self {
            proj: 0,
            axis: 1,
            mode: 0,
            cen_x: 0.,
            cen_y: 0.,
            cen_z: 0.,
            siz_x: 1.,
            siz_y: 1.,
            siz_z: 1.,
            rot_h: 0.,
            rot_p: 0.,
            rot_b: 0.,
            gaps: 0.2,
            seg_mat: false,
            part: false,
            orient: true,
            seam: 0.,
            nseg: 6,
            iter: 2,
            world: true,
            name: "Texture".into(),
            newmap: true,
        }
    }
}
impl TypedOp for CreateUvMapOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let kind = *ProjectionKind::ALL
            .get(self.proj as usize)
            .ok_or("Projection Type is out of range")?;
        let axis = match self.axis {
            0 => Axis3::X,
            1 => Axis3::Y,
            2 => Axis3::Z,
            _ => return Err("Axis must be 0 (X), 1 (Y) or 2 (Z)".into()),
        };
        let mut input = ctx.input.clone();
        let name = self.name.trim();
        let map = if name.is_empty() || (!self.newmap && !input.uv_maps.contains_key(name)) {
            crate::uv_tools::target_uv_map(&input, None)
        } else {
            name.to_owned()
        };
        if !input.uv_maps.contains_key(&map) {
            let rows = crate::uv_maps::empty_rows(&input);
            input.uv_maps.insert(map.clone(), rows);
        }
        let mut o = ProjectOptions {
            uv_map: Some(map),
            kind,
            axis,
            manual: self.mode == 1,
            size: DVec3::new(self.siz_x, self.siz_y, self.siz_z),
            center: DVec3::new(self.cen_x, self.cen_y, self.cen_z),
            rotation: DVec3::new(self.rot_p, self.rot_h, self.rot_b)
                * (180. / std::f64::consts::PI),
            seam_angle: self.seam.to_degrees(),
            ..Default::default()
        };
        o.atlas.fewer = kind == ProjectionKind::Atlas2;
        o.atlas.segment_by_material = self.seg_mat;
        o.atlas.segment_by_part = self.part;
        o.atlas.orientation = self.orient;
        o.atlas.segments = self.nseg;
        o.atlas.iterations = self.iter;
        o.atlas.relative_gap = self.gaps;
        let faces: Vec<u32> = ctx
            .chosen_or_all()?
            .into_iter()
            .filter(|&f| !input.is_polyline(f as usize))
            .collect();
        if faces.is_empty() {
            return Ok(input);
        }
        let (scene, _) = crate::meshop_scene::one_mesh_scene(ctx, &input, false)?;
        crate::uv_project::project_mesh(&scene, &faces, &o)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let names: Vec<&'static str> = ProjectionKind::ALL.iter().map(|k| k.label()).collect();
    vec![Arc::new(Typed::<CreateUvMapOp>::new(
        "uv.create.item",
        "Create UV Map",
        "UV",
        true,
        vec![
            Field::new("name", "UV Map", FieldKind::Text),
            check("newmap", "Create New Map"),
            choice("proj", "Projection Type", &names),
            choice("axis", "Axis", &["X", "Y", "Z"]),
            choice("mode", "Settings", &["Automatic", "Manual"]),
            distance("cenX", "Center X"),
            distance("cenY", "Center Y"),
            distance("cenZ", "Center Z"),
            distance("sizX", "Size X"),
            distance("sizY", "Size Y"),
            distance("sizZ", "Size Z"),
            Field::new("rotP", "Rotation X", FieldKind::Angle),
            Field::new("rotH", "Rotation Y", FieldKind::Angle),
            Field::new("rotB", "Rotation Z", FieldKind::Angle),
            Field::new("seam", "Seam Angle", FieldKind::Angle),
            Field::new("gaps", "Relative Gap Size", FieldKind::Percent),
            check("segMat", "Segment by Material"),
            check("part", "Segment by Part"),
            check("orient", "Orientation"),
            count("nseg", "Segments", 1, 10_000),
            count("iter", "Iterations", 0, 10_000),
            hidden("world"),
        ],
    ))]
}
