//! The Cube, Sphere and Cylinder as shapes: every option their tools show,
//! and the one generator that the tools and their MeshOps share (GitHub
//! #14, Jason 2026-11-25: one generator per primitive, so the two can't
//! drift). The other primitives already have one parameter struct and one
//! generator each (`ConeParameters` / `cone_geometry`, ...).
//!
//! A shape is built in its own space: centred on `position`, unmoved by
//! Symmetry or the target mesh's transform. The tools then double it for
//! Symmetry and move it into the mesh; a MeshOp appends it as it is.
//!
//! `falloff` says what a falloff's weight does to the shape (GitHub #15,
//! shape_falloff.rs); the weights come with [`CubeShape::mesh_weighted`]
//! and its siblings, from the tool falloff or the op's own copy.
use crate::{CylinderCap, Mesh, RadialParameters, Result, ShapeFalloff};
use glam::DVec3;

/// The Cube tool's shape.
#[derive(Clone, Debug, PartialEq)]
pub struct CubeShape {
    pub position: DVec3,
    /// Full widths along X, Y and Z.
    pub size: DVec3,
    pub segments: [u32; 3],
    /// The "Texture" UV map.
    pub make_uvs: bool,
    /// The spine a falloff shapes the cube along: 0 X, 1 Y, 2 Z.
    pub axis: usize,
    pub falloff: ShapeFalloff,
}
impl Default for CubeShape {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            size: DVec3::ONE,
            segments: [1; 3],
            make_uvs: true,
            axis: 1,
            falloff: ShapeFalloff::default(),
        }
    }
}
impl CubeShape {
    pub fn mesh(&self) -> Result<Mesh> {
        self.mesh_weighted(None)
    }
    /// The cube, shaped by `weight` (a position's falloff weight) when
    /// given. Follow Falloff spacing moves its segments.
    pub fn mesh_weighted(&self, weight: Weight) -> Result<Mesh> {
        let mut mesh = crate::segmented_cube(self.position, self.size, self.segments)?;
        if self.make_uvs && !mesh.faces.is_empty() {
            crate::add_cube_uvs(&mut mesh, self.position, self.size)?;
        }
        shaped(mesh, &self.falloff, self.position, self.axis, true, weight)
    }
}

/// The Sphere tool's shape.
#[derive(Clone, Debug, PartialEq)]
pub struct SphereShape {
    pub position: DVec3,
    /// World-named radii (semiaxes).
    pub radius: DVec3,
    pub sides: u32,
    pub segments: u32,
    /// The pole axis: 0 X, 1 Y, 2 Z.
    pub axis: usize,
    /// Sphere Mode: `None` is Globe, `Some(level)` Tessellated (PRIM-S-02).
    pub level: Option<u32>,
    pub make_uvs: bool,
    pub falloff: ShapeFalloff,
}
impl Default for SphereShape {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            radius: DVec3::splat(0.5),
            sides: 24,
            segments: 12,
            axis: 1,
            level: None,
            make_uvs: true,
            falloff: ShapeFalloff::default(),
        }
    }
}
impl SphereShape {
    fn radial(&self) -> RadialParameters {
        RadialParameters {
            radius: self.radius,
            sides: self.sides,
            segments: self.segments,
        }
    }
    pub fn mesh(&self) -> Result<Mesh> {
        self.mesh_weighted(None)
    }
    /// The sphere, shaped by `weight` when given. Its rings never move
    /// (Follow Falloff would change the sphere).
    pub fn mesh_weighted(&self, weight: Weight) -> Result<Mesh> {
        let mesh = self.plain()?;
        shaped(mesh, &self.falloff, self.position, self.axis, false, weight)
    }
    fn plain(&self) -> Result<Mesh> {
        if let Some(disc) = flat_disc(self.position, self.radial())? {
            return Ok(disc);
        }
        match self.level {
            Some(level) => crate::tessellated_sphere(
                self.position,
                self.radius,
                level,
                self.axis,
                self.make_uvs,
            ),
            None => {
                crate::globe_geometry_with(self.position, self.radial(), self.axis, self.make_uvs)
            }
        }
    }
}

/// The Cylinder tool's shape.
#[derive(Clone, Debug, PartialEq)]
pub struct CylinderShape {
    pub position: DVec3,
    /// World-named radii; the one along `axis` is half the height.
    pub radius: DVec3,
    pub sides: u32,
    pub segments: u32,
    /// 0 X, 1 Y, 2 Z.
    pub axis: usize,
    /// Bottom and top caps (PRIM-Y-01).
    pub caps: [CylinderCap; 2],
    pub make_uvs: bool,
    pub falloff: ShapeFalloff,
}
impl Default for CylinderShape {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            radius: DVec3::new(0.5, 0.75, 0.5),
            sides: 24,
            segments: 1,
            axis: 1,
            caps: [CylinderCap::Single; 2],
            make_uvs: true,
            falloff: ShapeFalloff::default(),
        }
    }
}
impl CylinderShape {
    pub fn mesh(&self) -> Result<Mesh> {
        self.mesh_weighted(None)
    }
    /// The cylinder, shaped by `weight` when given.
    pub fn mesh_weighted(&self, weight: Weight) -> Result<Mesh> {
        let mesh = self.plain()?;
        shaped(mesh, &self.falloff, self.position, self.axis, true, weight)
    }
    fn plain(&self) -> Result<Mesh> {
        let radial = RadialParameters {
            radius: self.radius,
            sides: self.sides,
            segments: self.segments,
        };
        if let Some(disc) = flat_disc(self.position, radial)? {
            return Ok(disc);
        }
        crate::cylinder_geometry_with(self.position, radial, self.caps, self.axis, self.make_uvs)
    }
}

/// A falloff weight per position, for the `mesh_weighted` generators.
pub type Weight<'a> = Option<&'a dyn Fn(glam::DVec3) -> f64>;

pub(crate) fn shaped(
    mut mesh: Mesh,
    falloff: &ShapeFalloff,
    centre: DVec3,
    axis: usize,
    respace: bool,
    weight: Weight,
) -> Result<Mesh> {
    falloff.validate()?;
    if let Some(weight) = weight {
        falloff.apply(&mut mesh, centre, axis, respace, weight)?;
    }
    Ok(mesh)
}

/// Drag creation's flat base (drag-creation-2026-11-06 rule 1): with exactly
/// one radius 0 the shape is one `sides`-gon in the other two axes, facing
/// that axis. A real disc that Drop keeps (rule 4).
fn flat_disc(position: DVec3, radial: RadialParameters) -> Result<Option<Mesh>> {
    let zero: Vec<usize> = (0..3).filter(|&a| radial.radius[a] == 0.).collect();
    let [normal] = zero[..] else {
        return Ok(None);
    };
    crate::flat_ngon(
        position,
        normal,
        radial.radius[(normal + 1) % 3],
        radial.radius[(normal + 2) % 3],
        radial.sides,
    )
    .map(Some)
}
