//! The primitive MeshOps (Create/Primitives): Cube, Sphere, Cylinder, Cone,
//! Capsule, Torus, Ellipsoid and Teapot, with the same options as their
//! tools (GitHub #14).
//!
//! Each op runs its tool's one generator (`CubeShape::mesh`,
//! `cone_geometry`, ...; Jason 2026-11-25): an op's values convert to the
//! tool's parameters and back with every field named, so a new tool option
//! doesn't build until its op has it too, and the two can't drift.
//!
//! Values use the ops' saved-name style (`cenX`, `sizeX`, camelCase);
//! distances in metres, percents as fractions (1.0 = 100%). Names the
//! reference map doesn't give are Operandi's own (provisional, PRIM-OP-Q01).
//! A tool-only setting (Bounds, Symmetry, the numeric UV map picker's
//! history) has no value.
use crate::mesh_stack_builtin::{check, choice, count, finite, xyz};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::teapot::TeapotParameters;
use crate::{
    CapsuleEnds, CapsuleParameters, ConeCap, ConeParameters, CubeShape, CylinderCap, CylinderShape,
    EllipsoidParameters, Mesh, PolygonRepresentation, Result, SphereShape, TorusParameters,
};
use crate::{FalloffSample, FalloffSpacing, ShapeFalloff, ShapeWeights};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A generator's result: the input with the new geometry appended, and
/// its groups (PG table): `all`, and by facing along the shape's axis
/// `capA` (Top), `capB` (Bottom), `caps` and `sides`.
pub(crate) fn generated(ctx: &OpContext, g: Mesh, axis: usize) -> Result<Mesh> {
    let mut out = ctx.input.clone();
    let first = out.faces.len() as u32;
    crate::append_mesh(&mut out, g)?;
    let new: Vec<u32> = (first..out.faces.len() as u32).collect();
    let axis = axis.min(2);
    let facing = |f: u32| -> f64 {
        let face = &out.faces[f as usize];
        let mut n = DVec3::ZERO;
        for i in 0..face.vertices.len() {
            let a = out.vertices[face.vertices[i] as usize];
            let b = out.vertices[face.vertices[(i + 1) % face.vertices.len()] as usize];
            n += a.cross(b);
        }
        n.normalize_or_zero()[axis]
    };
    let top: Vec<u32> = new
        .iter()
        .copied()
        .filter(|&f| facing(f) > 1. - 1e-9)
        .collect();
    let bottom: Vec<u32> = new
        .iter()
        .copied()
        .filter(|&f| facing(f) < -1. + 1e-9)
        .collect();
    let sides: Vec<u32> = new
        .iter()
        .copied()
        .filter(|f| !top.contains(f) && !bottom.contains(f))
        .collect();
    ctx.tag(&mut out, "all", new.iter().copied());
    ctx.tag(&mut out, "caps", top.iter().chain(&bottom).copied());
    ctx.tag(&mut out, "capA", top);
    ctx.tag(&mut out, "capB", bottom);
    ctx.tag(&mut out, "sides", sides);
    Ok(out)
}

fn vec3(x: f64, y: f64, z: f64, what: &str) -> Result<DVec3> {
    finite(&[x, y, z], what)?;
    Ok(DVec3::new(x, y, z))
}
fn axis_of(axis: u32) -> Result<usize> {
    if axis > 2 {
        return Err("Axis must be 0 (X), 1 (Y) or 2 (Z)".into());
    }
    Ok(axis as usize)
}
fn pick<T: Copy>(all: &[T], index: u32, what: &str) -> Result<T> {
    all.get(index as usize)
        .copied()
        .ok_or_else(|| format!("{what} must be 0 to {}", all.len() - 1))
}
fn index_of<T: PartialEq>(all: &[T], value: &T) -> u32 {
    all.iter().position(|v| v == value).unwrap_or(0) as u32
}
fn uv_map(name: &str) -> Option<String> {
    (!name.is_empty()).then(|| name.to_owned())
}

const CONE_CAPS: [ConeCap; 4] = [
    ConeCap::None,
    ConeCap::Single,
    ConeCap::QuadGrid,
    ConeCap::Radial,
];
/// Polygon Type: the solid representations (Polyline isn't a primitive's).
const POLYGONS: [PolygonRepresentation; 3] = [
    PolygonRepresentation::Face,
    PolygonRepresentation::Subdivs,
    PolygonRepresentation::CatmullClark,
];
const CAPSULE_ENDS: [CapsuleEnds; 2] = [CapsuleEnds::Polar, CapsuleEnds::QuadGrid];

/// The primitive ops' falloff values (#15; Cone, Capsule, Torus and
/// Ellipsoid #35). The falloff layers come from the Falloff step below
/// the op (meshop_tool_pipe.rs; one rule for the whole stack, Jason
/// 2026-11-26), so a saved stack evaluates the same everywhere. Positions
/// are the op's own (the item's local space), for Use World Transforms
/// on and off alike (provisional, PRIM-OP-Q04).
macro_rules! op_falloff {
    ($($op:ty),*) => {$(
        impl $op {
            fn drive(&self) -> Result<ShapeFalloff> {
                let f = ShapeFalloff {
                    scale: self.falloff_scale,
                    offset: vec3(
                        self.falloff_offset_x,
                        self.falloff_offset_y,
                        self.falloff_offset_z,
                        "Falloff Offset",
                    )?,
                    twist: self.falloff_twist.to_degrees(),
                    sample: pick(&FalloffSample::ALL, self.falloff_sample, "Falloff Sample At")?,
                    spacing: pick(&FalloffSpacing::ALL, self.falloff_spacing, "Falloff Spacing")?,
                };
                f.validate()?;
                Ok(f)
            }
        }
    )*};
}
op_falloff!(
    CubeOp,
    SphereOp,
    CylinderOp,
    ConeOp,
    CapsuleOp,
    TorusOp,
    EllipsoidOp
);

type Weigher = Option<Box<dyn Fn(DVec3) -> f64>>;
fn weigher(weights: Option<&ShapeWeights>) -> Result<Weigher> {
    let Some(w) = weights else {
        return Ok(None);
    };
    w.validate()?;
    let w = w.clone();
    Ok(Some(Box::new(move |p| w.weigher(glam::DMat4::IDENTITY)(p))))
}
fn as_weight(w: &Weigher) -> crate::primitive_shapes::Weight<'_> {
    w.as_deref().map(|f| f as &dyn Fn(DVec3) -> f64)
}

// ---- Cube ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CubeOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    #[serde(rename = "sizeX")]
    pub size_x: f64,
    #[serde(rename = "sizeY")]
    pub size_y: f64,
    #[serde(rename = "sizeZ")]
    pub size_z: f64,
    #[serde(rename = "segmentsX")]
    pub segments_x: u32,
    #[serde(rename = "segmentsY")]
    pub segments_y: u32,
    #[serde(rename = "segmentsZ")]
    pub segments_z: u32,
    /// The spine a falloff shapes the cube along.
    pub axis: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    /// What the op's falloff does (#15); see [`OpFalloff`].
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl CubeOp {
    pub fn shape(&self) -> Result<CubeShape> {
        Ok(CubeShape {
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            size: vec3(self.size_x, self.size_y, self.size_z, "Size")?,
            segments: [self.segments_x, self.segments_y, self.segments_z],
            make_uvs: self.make_uvs,
            axis: axis_of(self.axis)?,
            falloff: self.drive()?,
        })
    }
}
impl From<&CubeShape> for CubeOp {
    fn from(s: &CubeShape) -> Self {
        let CubeShape {
            position: p,
            size,
            segments: [sx, sy, sz],
            make_uvs,
            axis,
            falloff,
        } = *s;
        Self {
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size_x: size.x,
            size_y: size.y,
            size_z: size.z,
            segments_x: sx,
            segments_y: sy,
            segments_z: sz,
            axis: axis as u32,
            make_uvs,
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
        }
    }
}
impl Default for CubeOp {
    fn default() -> Self {
        Self::from(&CubeShape::default())
    }
}
impl TypedOp for CubeOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let s = self.shape()?;
        let w = weigher(ctx.falloff)?;
        generated(ctx, s.mesh_weighted(as_weight(&w))?, 1)
    }
}

// ---- Sphere ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SphereOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    /// Radii (semiaxes).
    #[serde(rename = "sizeX")]
    pub size_x: f64,
    #[serde(rename = "sizeY")]
    pub size_y: f64,
    #[serde(rename = "sizeZ")]
    pub size_z: f64,
    pub sides: u32,
    pub segments: u32,
    pub axis: u32,
    /// Sphere Mode: 0 Globe, 1 Tessellated.
    pub method: u32,
    /// Tessellated's Subdivision Level.
    pub level: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    /// What the op's falloff does (#15); see [`OpFalloff`].
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl SphereOp {
    pub fn shape(&self) -> Result<SphereShape> {
        if self.method > 1 {
            return Err("Sphere Mode must be 0 (Globe) or 1 (Tessellated)".into());
        }
        Ok(SphereShape {
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            radius: vec3(self.size_x, self.size_y, self.size_z, "Radius")?,
            sides: self.sides,
            segments: self.segments,
            axis: axis_of(self.axis)?,
            level: (self.method == 1).then_some(self.level),
            make_uvs: self.make_uvs,
            falloff: self.drive()?,
        })
    }
}
impl From<&SphereShape> for SphereOp {
    fn from(s: &SphereShape) -> Self {
        let SphereShape {
            position: p,
            radius: r,
            sides,
            segments,
            axis,
            level,
            make_uvs,
            falloff,
        } = *s;
        Self {
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size_x: r.x,
            size_y: r.y,
            size_z: r.z,
            sides,
            segments,
            axis: axis as u32,
            method: u32::from(level.is_some()),
            level: level.unwrap_or(0),
            make_uvs,
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
        }
    }
}
impl Default for SphereOp {
    fn default() -> Self {
        Self::from(&SphereShape::default())
    }
}
impl TypedOp for SphereOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let s = self.shape()?;
        let w = weigher(ctx.falloff)?;
        generated(ctx, s.mesh_weighted(as_weight(&w))?, s.axis)
    }
}

// ---- Cylinder ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CylinderOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    /// Radii; the one along the axis is half the height.
    #[serde(rename = "sizeX")]
    pub size_x: f64,
    #[serde(rename = "sizeY")]
    pub size_y: f64,
    #[serde(rename = "sizeZ")]
    pub size_z: f64,
    pub sides: u32,
    pub segments: u32,
    pub axis: u32,
    /// Index into `CylinderCap::ALL`.
    pub cap_top: u32,
    pub cap_bottom: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    /// What the op's falloff does (#15); see [`OpFalloff`].
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl CylinderOp {
    pub fn shape(&self) -> Result<CylinderShape> {
        Ok(CylinderShape {
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            radius: vec3(self.size_x, self.size_y, self.size_z, "Radius")?,
            sides: self.sides,
            segments: self.segments,
            axis: axis_of(self.axis)?,
            caps: [
                pick(&CylinderCap::ALL, self.cap_bottom, "Cap Bottom")?,
                pick(&CylinderCap::ALL, self.cap_top, "Cap Top")?,
            ],
            make_uvs: self.make_uvs,
            falloff: self.drive()?,
        })
    }
}
impl From<&CylinderShape> for CylinderOp {
    fn from(s: &CylinderShape) -> Self {
        let CylinderShape {
            position: p,
            radius: r,
            sides,
            segments,
            axis,
            caps: [bottom, top],
            make_uvs,
            falloff,
        } = *s;
        Self {
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size_x: r.x,
            size_y: r.y,
            size_z: r.z,
            sides,
            segments,
            axis: axis as u32,
            cap_top: index_of(&CylinderCap::ALL, &top),
            cap_bottom: index_of(&CylinderCap::ALL, &bottom),
            make_uvs,
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
        }
    }
}
impl Default for CylinderOp {
    fn default() -> Self {
        Self::from(&CylinderShape::default())
    }
}
impl TypedOp for CylinderOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let s = self.shape()?;
        let w = weigher(ctx.falloff)?;
        generated(ctx, s.mesh_weighted(as_weight(&w))?, s.axis)
    }
}

// ---- Cone ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConeOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    /// Half extents: the base radii and half the height along the axis.
    #[serde(rename = "sizeX")]
    pub size_x: f64,
    #[serde(rename = "sizeY")]
    pub size_y: f64,
    #[serde(rename = "sizeZ")]
    pub size_z: f64,
    pub sides: u32,
    pub segments: u32,
    /// Polygon Type: 0 Face, 1 Subdivs, 2 Catmull-Clark.
    pub pol_type: u32,
    /// Index into Top/Bottom Cap's None, Single Polygon, Quad Grid, Radial.
    pub cap_top: u32,
    pub cap_bottom: u32,
    /// Fraction (1.0 = 100%).
    pub edge_weight: f64,
    pub axis: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    pub compensation: bool,
    /// Fraction (0.07 = 7%).
    pub gaps: f64,
    /// The UV map to write; empty = the primitive map.
    #[serde(rename = "uvMap")]
    pub uv_map: String,
    /// What the op's falloff does (#35, as #15); see `op_falloff!`.
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl ConeOp {
    pub fn parameters(&self) -> Result<ConeParameters> {
        finite(&[self.edge_weight, self.gaps], "Edge Weight and Gaps")?;
        Ok(ConeParameters {
            falloff: self.drive()?,
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            size: vec3(self.size_x, self.size_y, self.size_z, "Size")?,
            axis: axis_of(self.axis)?,
            sides: self.sides,
            segments: self.segments,
            polygon: pick(&POLYGONS, self.pol_type, "Polygon Type")?,
            top_cap: pick(&CONE_CAPS, self.cap_top, "Top Cap")?,
            bottom_cap: pick(&CONE_CAPS, self.cap_bottom, "Bottom Cap")?,
            edge_weight: self.edge_weight * 100.,
            make_uvs: self.make_uvs,
            compensation: self.compensation,
            gaps: self.gaps * 100.,
            selected_uv_map: uv_map(&self.uv_map),
        })
    }
}
impl From<&ConeParameters> for ConeOp {
    fn from(c: &ConeParameters) -> Self {
        let ConeParameters {
            falloff,
            position: p,
            size: s,
            axis,
            sides,
            segments,
            polygon,
            top_cap,
            bottom_cap,
            edge_weight,
            make_uvs,
            compensation,
            gaps,
            selected_uv_map,
        } = c;
        Self {
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size_x: s.x,
            size_y: s.y,
            size_z: s.z,
            sides: *sides,
            segments: *segments,
            pol_type: index_of(&POLYGONS, polygon),
            cap_top: index_of(&CONE_CAPS, top_cap),
            cap_bottom: index_of(&CONE_CAPS, bottom_cap),
            edge_weight: edge_weight / 100.,
            axis: *axis as u32,
            make_uvs: *make_uvs,
            compensation: *compensation,
            gaps: gaps / 100.,
            uv_map: selected_uv_map.clone().unwrap_or_default(),
        }
    }
}
impl Default for ConeOp {
    fn default() -> Self {
        Self::from(&ConeParameters::default())
    }
}
impl TypedOp for ConeOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let p = self.parameters()?;
        // #35: shaped by the op's own falloff copy, as the tool is by the
        // tool falloff.
        let w = weigher(ctx.falloff)?;
        let mesh = crate::primitive_shapes::shaped(
            crate::cone_geometry(&p)?,
            &p.falloff,
            p.position,
            p.axis,
            true,
            as_weight(&w),
        )?;
        generated(ctx, mesh, p.axis)
    }
}

// ---- Capsule ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CapsuleOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    /// Half the length along the axis and the radii across it.
    #[serde(rename = "sizeX")]
    pub size_x: f64,
    #[serde(rename = "sizeY")]
    pub size_y: f64,
    #[serde(rename = "sizeZ")]
    pub size_z: f64,
    pub sides: u32,
    pub segments: u32,
    pub end_segments: u32,
    /// Each rounded end's height, metres.
    pub end_size: f64,
    /// 0 Polar, 1 Quad Grid.
    pub ends: u32,
    pub bulge: f64,
    pub axis: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    /// What the op's falloff does (#35, as #15); see `op_falloff!`.
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl CapsuleOp {
    pub fn parameters(&self) -> Result<CapsuleParameters> {
        finite(&[self.end_size, self.bulge], "End Size and Bulge")?;
        Ok(CapsuleParameters {
            falloff: self.drive()?,
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            radius: vec3(self.size_x, self.size_y, self.size_z, "Radius")?,
            sides: self.sides,
            segments: self.segments,
            end_segments: self.end_segments,
            end_size: self.end_size,
            ends: pick(&CAPSULE_ENDS, self.ends, "Ends")?,
            bulge: self.bulge,
            axis: axis_of(self.axis)?,
            make_uvs: self.make_uvs,
        }
        .normalized())
    }
}
impl From<&CapsuleParameters> for CapsuleOp {
    fn from(c: &CapsuleParameters) -> Self {
        let CapsuleParameters {
            falloff,
            position: p,
            radius: r,
            sides,
            segments,
            end_segments,
            end_size,
            ends,
            bulge,
            axis,
            make_uvs,
        } = *c;
        Self {
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size_x: r.x,
            size_y: r.y,
            size_z: r.z,
            sides,
            segments,
            end_segments,
            end_size,
            ends: index_of(&CAPSULE_ENDS, &ends),
            bulge,
            axis: axis as u32,
            make_uvs,
        }
    }
}
impl Default for CapsuleOp {
    fn default() -> Self {
        Self::from(&CapsuleParameters::default())
    }
}
impl TypedOp for CapsuleOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let p = self.parameters()?;
        // #35: shaped by the op's own falloff copy, as the tool is by the
        // tool falloff.
        let w = weigher(ctx.falloff)?;
        let mesh = crate::primitive_shapes::shaped(
            crate::capsule_geometry(&p)?,
            &p.falloff,
            p.position,
            p.axis,
            false,
            as_weight(&w),
        )?;
        generated(ctx, mesh, p.axis)
    }
}

// ---- Torus ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TorusOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    pub sides: u32,
    pub segments: u32,
    /// Ring Radius, Cross Section and Height: the stored sizes (NPR-TOR-03).
    pub ring_radius: f64,
    pub cross_section: f64,
    pub height: f64,
    pub bulge_top: f64,
    pub bulge_side: f64,
    pub axis: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    /// What the op's falloff does (#35, as #15); see `op_falloff!`.
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl TorusOp {
    pub fn parameters(&self) -> Result<TorusParameters> {
        finite(
            &[
                self.ring_radius,
                self.cross_section,
                self.height,
                self.bulge_top,
                self.bulge_side,
            ],
            "Torus sizes and Bulge",
        )?;
        Ok(TorusParameters {
            falloff: self.drive()?,
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            sides: self.sides,
            segments: self.segments,
            ring_radius: self.ring_radius,
            cross_section: self.cross_section,
            height: self.height,
            bulge_top: self.bulge_top,
            bulge_side: self.bulge_side,
            axis: axis_of(self.axis)?,
            make_uvs: self.make_uvs,
        }
        .normalized())
    }
}
impl From<&TorusParameters> for TorusOp {
    fn from(t: &TorusParameters) -> Self {
        let TorusParameters {
            falloff,
            position: p,
            sides,
            segments,
            ring_radius,
            cross_section,
            height,
            bulge_top,
            bulge_side,
            axis,
            make_uvs,
        } = *t;
        Self {
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            sides,
            segments,
            ring_radius,
            cross_section,
            height,
            bulge_top,
            bulge_side,
            axis: axis as u32,
            make_uvs,
        }
    }
}
impl Default for TorusOp {
    fn default() -> Self {
        Self::from(&TorusParameters::default())
    }
}
impl TypedOp for TorusOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let p = self.parameters()?;
        // #35: shaped by the op's own falloff copy, as the tool is by the
        // tool falloff.
        let w = weigher(ctx.falloff)?;
        let mesh = crate::primitive_shapes::shaped(
            crate::torus_geometry(&p)?,
            &p.falloff,
            p.position,
            p.axis,
            false,
            as_weight(&w),
        )?;
        generated(ctx, mesh, p.axis)
    }
}

// ---- Ellipsoid ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EllipsoidOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    #[serde(rename = "sizeX")]
    pub size_x: f64,
    #[serde(rename = "sizeY")]
    pub size_y: f64,
    #[serde(rename = "sizeZ")]
    pub size_z: f64,
    pub sides: u32,
    pub segments: u32,
    /// Polygon Type: 0 Face, 1 Subdivs, 2 Catmull-Clark.
    pub pol_type: u32,
    pub bulge_top: f64,
    pub bulge_side: f64,
    pub axis: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
    pub compensation: bool,
    /// Fraction (0.07 = 7%).
    pub gaps: f64,
    #[serde(rename = "uvMap")]
    pub uv_map: String,
    /// What the op's falloff does (#35, as #15); see `op_falloff!`.
    #[serde(rename = "falloffScale")]
    pub falloff_scale: f64,
    #[serde(rename = "falloffOffsetX")]
    pub falloff_offset_x: f64,
    #[serde(rename = "falloffOffsetY")]
    pub falloff_offset_y: f64,
    #[serde(rename = "falloffOffsetZ")]
    pub falloff_offset_z: f64,
    /// Radians.
    #[serde(rename = "falloffTwist")]
    pub falloff_twist: f64,
    /// 0 Spine, 1 Points.
    #[serde(rename = "falloffSample")]
    pub falloff_sample: u32,
    /// 0 Even, 1 Follow Falloff.
    #[serde(rename = "falloffSpacing")]
    pub falloff_spacing: u32,
    /// An old scene's own copy of the falloff layers (#15): read and
    /// ignored, never saved; the Falloff step below shapes the op now.
    #[serde(default, skip_serializing)]
    pub falloff: Option<serde_json::Value>,
}
impl EllipsoidOp {
    pub fn parameters(&self) -> Result<EllipsoidParameters> {
        finite(
            &[self.bulge_top, self.bulge_side, self.gaps],
            "Bulge and Gaps",
        )?;
        Ok(EllipsoidParameters {
            falloff: self.drive()?,
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            radius: vec3(self.size_x, self.size_y, self.size_z, "Radius")?,
            sides: self.sides,
            segments: self.segments,
            polygon: pick(&POLYGONS, self.pol_type, "Polygon Type")?,
            bulge_top: self.bulge_top,
            bulge_side: self.bulge_side,
            axis: axis_of(self.axis)?,
            make_uvs: self.make_uvs,
            compensation: self.compensation,
            gaps: self.gaps * 100.,
            selected_uv_map: uv_map(&self.uv_map),
            symmetry_axis: None,
        }
        .normalized())
    }
}
impl From<&EllipsoidParameters> for EllipsoidOp {
    fn from(e: &EllipsoidParameters) -> Self {
        let EllipsoidParameters {
            falloff,
            position: p,
            radius: r,
            sides,
            segments,
            polygon,
            bulge_top,
            bulge_side,
            axis,
            make_uvs,
            compensation,
            gaps,
            selected_uv_map,
            // The tool's Numeric Symmetry: the op stack has its own
            // Symmetrize op instead.
            symmetry_axis: _,
        } = e;
        Self {
            falloff_scale: falloff.scale,
            falloff_offset_x: falloff.offset.x,
            falloff_offset_y: falloff.offset.y,
            falloff_offset_z: falloff.offset.z,
            falloff_twist: falloff.twist.to_radians(),
            falloff_sample: index_of(&FalloffSample::ALL, &falloff.sample),
            falloff_spacing: index_of(&FalloffSpacing::ALL, &falloff.spacing),
            falloff: None,
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size_x: r.x,
            size_y: r.y,
            size_z: r.z,
            sides: *sides,
            segments: *segments,
            pol_type: index_of(&POLYGONS, polygon),
            bulge_top: *bulge_top,
            bulge_side: *bulge_side,
            axis: *axis as u32,
            make_uvs: *make_uvs,
            compensation: *compensation,
            gaps: gaps / 100.,
            uv_map: selected_uv_map.clone().unwrap_or_default(),
        }
    }
}
impl Default for EllipsoidOp {
    fn default() -> Self {
        Self::from(&EllipsoidParameters::default())
    }
}
impl TypedOp for EllipsoidOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let p = self.parameters()?;
        // #35: shaped by the op's own falloff copy, as the tool is by the
        // tool falloff.
        let w = weigher(ctx.falloff)?;
        let mesh = crate::primitive_shapes::shaped(
            crate::ellipsoid_geometry(&p)?,
            &p.falloff,
            p.position,
            p.axis,
            false,
            as_weight(&w),
        )?;
        generated(ctx, mesh, p.axis)
    }
}

// ---- Teapot ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TeapotOp {
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    /// The longest side, spout tip to handle.
    pub size: f64,
    pub axis: u32,
    #[serde(rename = "makeUVs")]
    pub make_uvs: bool,
}
impl TeapotOp {
    pub fn parameters(&self) -> Result<TeapotParameters> {
        finite(&[self.size], "Size")?;
        Ok(TeapotParameters {
            position: vec3(self.cen_x, self.cen_y, self.cen_z, "Position")?,
            size: self.size,
            axis: axis_of(self.axis)?,
            make_uvs: self.make_uvs,
        })
    }
}
impl From<&TeapotParameters> for TeapotOp {
    fn from(t: &TeapotParameters) -> Self {
        let TeapotParameters {
            position: p,
            size,
            axis,
            make_uvs,
        } = *t;
        Self {
            cen_x: p.x,
            cen_y: p.y,
            cen_z: p.z,
            size,
            axis: axis as u32,
            make_uvs,
        }
    }
}
impl Default for TeapotOp {
    fn default() -> Self {
        Self::from(&TeapotParameters::default())
    }
}
impl TypedOp for TeapotOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let p = self.parameters()?;
        generated(ctx, crate::teapot::teapot_geometry(&p)?, p.axis)
    }
}

// ---- the table entries ----

const PRIMITIVE_GROUPS: &[(&str, &str)] = &[
    ("all", "All"),
    ("caps", "Caps"),
    ("capA", "Top"),
    ("capB", "Bottom"),
    ("sides", "Sides"),
];
const BIG: u32 = 100_000;

fn position() -> [Field; 3] {
    xyz("cen", "Position", FieldKind::Distance)
}
fn size(label: &'static str) -> [Field; 3] {
    xyz("size", label, FieldKind::Distance)
}
fn axis() -> Field {
    choice("axis", "Axis", &["X", "Y", "Z"])
}
fn make_uvs() -> Field {
    check("makeUVs", "Make UVs")
}
fn number(key: &'static str, label: &'static str) -> Field {
    Field::new(key, label, FieldKind::Number)
}
fn polygon_type() -> Field {
    choice(
        "polType",
        "Polygon Type",
        &["Face", "Subdivs", "Catmull-Clark"],
    )
}
/// The Falloff rows (#15): what the op's own falloff copy drives.
fn falloff_fields() -> Vec<Field> {
    let mut f = vec![Field::new(
        "falloffScale",
        "Falloff Scale",
        FieldKind::Percent,
    )];
    f.extend(xyz("falloffOffset", "Falloff Offset", FieldKind::Distance));
    f.extend([
        Field::new("falloffTwist", "Falloff Twist", FieldKind::Angle),
        choice("falloffSample", "Sample At", &["Spine", "Points"]),
        choice("falloffSpacing", "Spacing", &["Even", "Follow Falloff"]),
    ]);
    f
}
/// The Cone's and Ellipsoid's UV rows.
fn uv_rows() -> [Field; 4] {
    [
        make_uvs(),
        check("compensation", "Distortion Compensation"),
        Field::new("gaps", "Gaps", FieldKind::Percent),
        Field::new("uvMap", "UV Map", FieldKind::Text),
    ]
}

/// The primitive ops, in Add Operator order (the Create tab's order), with
/// their tools' rows in the tools' order.
pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    const FOLDER: &str = "Create/Primitives";
    let cylinder_caps: Vec<&'static str> = CylinderCap::ALL.iter().map(|c| c.label()).collect();
    let cone_caps = ["None", "Single Polygon", "Quad Grid", "Radial"];

    let mut cube = position().to_vec();
    cube.extend(size("Size"));
    cube.extend(xyz(
        "segments",
        "Segments",
        FieldKind::Count { min: 1, max: BIG },
    ));
    cube.extend([axis(), make_uvs()]);
    cube.extend(falloff_fields());

    let mut sphere = position().to_vec();
    sphere.extend(size("Radius"));
    sphere.extend([
        count("sides", "Sides", 3, BIG),
        count("segments", "Segments", 2, BIG),
        // Quad Ball isn't built, so it's left out (UIC-117).
        choice("method", "Sphere Mode", &["Globe", "Tessellated"]),
        count("level", "Subdivision Level", 0, 6),
        axis(),
        make_uvs(),
    ]);
    sphere.extend(falloff_fields());

    let mut cylinder = position().to_vec();
    cylinder.extend(size("Radius"));
    cylinder.extend([
        count("sides", "Sides", 3, BIG),
        count("segments", "Segments", 1, BIG),
        choice("capTop", "Cap Top", &cylinder_caps),
        choice("capBottom", "Cap Bottom", &cylinder_caps),
        axis(),
        make_uvs(),
    ]);
    cylinder.extend(falloff_fields());

    let mut cone = position().to_vec();
    cone.extend(size("Size"));
    cone.extend([
        count("sides", "Sides", 3, 4096),
        count("segments", "Segments", 1, BIG),
        polygon_type(),
        choice("capTop", "Top Cap", &cone_caps),
        choice("capBottom", "Bottom Cap", &cone_caps),
        Field::new("edgeWeight", "Edge Weight", FieldKind::Percent),
        axis(),
    ]);
    cone.extend(uv_rows());
    cone.extend(falloff_fields());

    let mut capsule = position().to_vec();
    capsule.extend(size("Radius"));
    capsule.extend([
        count("sides", "Sides", 3, BIG),
        count("segments", "Segments", 1, BIG),
        count("endSegments", "End Segments", 1, BIG),
        Field::new("endSize", "End Size", FieldKind::Distance),
        choice("ends", "Ends", &["Polar", "Quad Grid"]),
        number("bulge", "Bulge"),
        axis(),
        make_uvs(),
    ]);
    capsule.extend(falloff_fields());

    let mut torus = position().to_vec();
    torus.extend([
        count("sides", "Sides", 3, BIG),
        count("segments", "Segments", 3, BIG),
        Field::new("ringRadius", "Ring Radius", FieldKind::Distance),
        Field::new("crossSection", "Cross Section", FieldKind::Distance),
        Field::new("height", "Height", FieldKind::Distance),
        number("bulgeTop", "Bulge Top"),
        number("bulgeSide", "Bulge Side"),
        axis(),
        make_uvs(),
    ]);
    torus.extend(falloff_fields());

    let mut ellipsoid = position().to_vec();
    ellipsoid.extend(size("Radius"));
    ellipsoid.extend([
        count("sides", "Sides", 3, BIG),
        count("segments", "Segments", 2, BIG),
        polygon_type(),
        number("bulgeTop", "Bulge Top"),
        number("bulgeSide", "Bulge Side"),
        axis(),
    ]);
    ellipsoid.extend(uv_rows());
    ellipsoid.extend(falloff_fields());

    let mut teapot = position().to_vec();
    teapot.extend([
        Field::new("size", "Size", FieldKind::Distance),
        axis(),
        make_uvs(),
    ]);

    fn entry<T: TypedOp>(
        id: &'static str,
        label: &'static str,
        fields: Vec<Field>,
    ) -> Arc<dyn OpDef> {
        Arc::new(Typed::<T>::new(id, label, FOLDER, false, fields).with_groups(PRIMITIVE_GROUPS))
    }
    vec![
        entry::<CubeOp>("prim.cube.item", "Cube", cube),
        entry::<SphereOp>("prim.sphere.item", "Sphere", sphere),
        entry::<EllipsoidOp>("prim.ellipsoid.item", "Ellipsoid", ellipsoid),
        entry::<ConeOp>("prim.cone.item", "Cone", cone),
        entry::<TorusOp>("prim.toroid.item", "Torus", torus),
        entry::<CylinderOp>("prim.cylinder.item", "Cylinder", cylinder),
        entry::<CapsuleOp>("prim.capsule.item", "Capsule", capsule),
        entry::<TeapotOp>("prim.teapot.item", "Teapot", teapot),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meshop_types::op_def;

    fn run(id: &str, values: serde_json::Value) -> Result<Mesh> {
        let def = op_def(id).expect("op type");
        let ctx = OpContext {
            input: &Mesh::default(),
            op: 1,
            selection: None,
            element: Default::default(),
            links: &[],
            time: Default::default(),
            notes: Default::default(),
            falloff: None,
        };
        def.run(&ctx, &values)
    }
    fn with(id: &str, changes: serde_json::Value) -> serde_json::Value {
        let mut v = op_def(id).unwrap().defaults();
        for (k, x) in changes.as_object().unwrap() {
            assert!(v.get(k).is_some(), "{id} has no value {k}");
            v[k] = x.clone();
        }
        v
    }
    /// Points and polygons, without the op's group tags.
    fn same(a: &Mesh, b: &Mesh) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.faces.len(), b.faces.len());
        for (x, y) in a.faces.iter().zip(b.faces.iter()) {
            assert_eq!(x.vertices, y.vertices);
        }
        assert_eq!(a.uv_maps, b.uv_maps);
    }

    const IDS: [&str; 8] = [
        "prim.cube.item",
        "prim.sphere.item",
        "prim.ellipsoid.item",
        "prim.cone.item",
        "prim.toroid.item",
        "prim.cylinder.item",
        "prim.capsule.item",
        "prim.teapot.item",
    ];

    #[test]
    fn every_primitive_op_value_has_a_form_field_and_reads_back() {
        for id in IDS {
            let def = op_def(id).unwrap();
            let defaults = def.defaults();
            let keys: Vec<&String> = defaults.as_object().unwrap().keys().collect();
            for k in &keys {
                assert!(
                    def.fields().iter().any(|f| f.key == k.as_str()),
                    "{id}: {k} has no field"
                );
            }
            assert_eq!(
                def.fields().len(),
                keys.len(),
                "{id}: a field with no value"
            );
            assert_eq!(def.read(1, &defaults).unwrap(), defaults);
            assert!(
                !run(id, defaults).unwrap().faces.is_empty(),
                "{id} makes polygons"
            );
        }
    }

    #[test]
    fn ops_and_tools_build_the_same_mesh_for_every_option() {
        // Sphere: Tessellated, along X, no UVs, as the tool builds it.
        let shape = SphereShape {
            axis: 0,
            level: Some(2),
            make_uvs: false,
            radius: DVec3::new(0.5, 1., 0.25),
            ..Default::default()
        };
        let op = run(
            "prim.sphere.item",
            serde_json::to_value(SphereOp::from(&shape)).unwrap(),
        )
        .unwrap();
        same(&op, &shape.mesh().unwrap());

        let shape = CylinderShape {
            axis: 2,
            caps: [CylinderCap::Radial, CylinderCap::None],
            segments: 3,
            ..Default::default()
        };
        let op = run(
            "prim.cylinder.item",
            with(
                "prim.cylinder.item",
                serde_json::json!({
                    "axis": 2, "capTop": 0, "capBottom": 3, "segments": 3
                }),
            ),
        )
        .unwrap();
        same(&op, &shape.mesh().unwrap());

        let shape = CubeShape {
            segments: [2, 3, 1],
            make_uvs: true,
            ..Default::default()
        };
        let op = run(
            "prim.cube.item",
            with(
                "prim.cube.item",
                serde_json::json!({"segmentsX": 2, "segmentsY": 3}),
            ),
        )
        .unwrap();
        same(&op, &shape.mesh().unwrap());

        let cone = ConeParameters {
            bottom_cap: ConeCap::QuadGrid,
            sides: 8,
            segments: 2,
            axis: 0,
            ..Default::default()
        };
        let op = run(
            "prim.cone.item",
            serde_json::to_value(ConeOp::from(&cone)).unwrap(),
        )
        .unwrap();
        same(&op, &crate::cone_geometry(&cone).unwrap());

        let torus = TorusParameters {
            bulge_top: 0.3,
            axis: 2,
            ..Default::default()
        };
        let op = run(
            "prim.toroid.item",
            serde_json::to_value(TorusOp::from(&torus)).unwrap(),
        )
        .unwrap();
        same(&op, &crate::torus_geometry(&torus.normalized()).unwrap());
    }

    #[test]
    fn values_round_trip_through_the_tool_parameters() {
        let cone = ConeParameters {
            edge_weight: 50.,
            gaps: 25.,
            selected_uv_map: Some("Second".into()),
            ..Default::default()
        };
        assert_eq!(ConeOp::from(&cone).parameters().unwrap(), cone);
        let e = EllipsoidParameters {
            gaps: 12.5,
            polygon: PolygonRepresentation::CatmullClark,
            ..Default::default()
        };
        assert_eq!(EllipsoidOp::from(&e).parameters().unwrap(), e.normalized());
        let s = SphereShape {
            level: Some(3),
            ..Default::default()
        };
        assert_eq!(SphereOp::from(&s).shape().unwrap(), s);
    }

    #[test]
    fn out_of_range_choices_are_refused_not_panics() {
        for (id, key) in [
            ("prim.sphere.item", "method"),
            ("prim.cylinder.item", "capTop"),
            ("prim.cone.item", "polType"),
            ("prim.capsule.item", "ends"),
            ("prim.teapot.item", "axis"),
        ] {
            assert!(
                run(id, with(id, serde_json::json!({ key: 9 }))).is_err(),
                "{id} {key}"
            );
        }
        assert!(
            run(
                "prim.cube.item",
                with("prim.cube.item", serde_json::json!({"segmentsX": 0}))
            )
            .is_err()
        );
    }

    #[test]
    fn caps_groups_follow_the_axis() {
        let values = with("prim.cylinder.item", serde_json::json!({"axis": 0}));
        let mesh = run("prim.cylinder.item", values).unwrap();
        let top = mesh
            .attributes
            .polygon_sets
            .get(&crate::meshop_selection::group_set(1, "capA"))
            .unwrap();
        assert_eq!(top.len(), 1, "one Single Polygon cap facing +X");
    }
}
