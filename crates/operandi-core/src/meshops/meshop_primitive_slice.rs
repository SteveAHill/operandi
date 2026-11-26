//! Primitive Slice as a Mesh Stack op (`prim.slice.item`; the reference
//! saves a Content Preset + Slice Effector + Primitive Generator,
//! TYPE-IDS.md): a rectangle, circle or ellipse, placed by its Center and
//! Axis, is cut into the polygons as the Primitive Slice tool cuts its
//! shape (PS-40..45), with the tool's corner rounding, rotation, flips and
//! clone grid. The cutter starts on Action Axis, which here is the shape's
//! own Axis.
//!
//! Provisional (MOQ-65): Type lists Rectangle, Circle and Ellipse (Preset
//! waits for the preset browser, so the Content Preset's channels aren't
//! kept); the shape's Axis is always the Axis X/Y/Z vector (the
//! generator's axis choice and its handles, Lazy Apply and Exponent only
//! matter to the tool and aren't kept).
use crate::mesh_stack_builtin::{check, choice, count, distance, finite};
use crate::meshop_cutter_common::{ACTION_AXIS, cutter_fields, cutter_op};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::path_slice::{PrimitiveShape, PrimitiveSlice, ShapePlace};
use crate::{Mesh, Result};
use glam::DVec3;
use std::sync::Arc;

/// Type choices, by position.
pub const SHAPES: [&str; 3] = ["Rectangle", "Circle", "Ellipse"];

cutter_op! {
    pub struct PrimitiveSliceOp {
        /// Type, by position in [`SHAPES`].
        #[serde(rename = "type")]
        shape: u32 = 0,
        #[serde(rename = "centX")]
        cent_x: f64 = 0.,
        #[serde(rename = "centY")]
        cent_y: f64 = 0.,
        #[serde(rename = "centZ")]
        cent_z: f64 = 0.,
        /// The shape's Axis (its plane's normal).
        vec_x: f64 = 0.,
        vec_y: f64 = 0.,
        vec_z: f64 = 1.,
        /// Half Size X and Y (Rectangle, Ellipse).
        size_x: f64 = 0.5,
        size_y: f64 = 0.5,
        /// Circle.
        radius: f64 = 0.5,
        sides: u32 = 24,
        /// Rectangle's Corner Radius and Corner Segments.
        rect_radius: f64 = 0.,
        rect_segs: u32 = 8,
        /// Radians.
        heading: f64 = 0.,
        pitch: f64 = 0.,
        bank: f64 = 0.,
        flip_x: bool = false,
        flip_y: bool = false,
        clone_x: u32 = 1,
        clone_y: u32 = 1,
        clone_gap_x: f64 = 0.01,
        clone_gap_y: f64 = 0.01,
        clone_center: bool = false,
    }
    axis = ACTION_AXIS;
}

impl PrimitiveSliceOp {
    /// The tool settings these values describe.
    fn shape(&self) -> Result<PrimitiveSlice> {
        finite(
            &[
                self.cent_x,
                self.cent_y,
                self.cent_z,
                self.vec_x,
                self.vec_y,
                self.vec_z,
                self.size_x,
                self.size_y,
                self.radius,
                self.rect_radius,
                self.heading,
                self.pitch,
                self.bank,
                self.clone_gap_x,
                self.clone_gap_y,
            ],
            "Primitive Slice values",
        )?;
        let shape = match self.shape {
            0 => PrimitiveShape::Rectangle,
            1 => PrimitiveShape::Circle,
            2 => PrimitiveShape::Ellipse,
            _ => return Err("Type must be 0 (Rectangle) to 2 (Ellipse)".into()),
        };
        let normal = DVec3::new(self.vec_x, self.vec_y, self.vec_z);
        if normal.length() < 1e-12 {
            return Err("The shape's Axis needs a direction".into());
        }
        let size = if shape == PrimitiveShape::Circle {
            self.radius
        } else {
            self.size_x.min(self.size_y)
        };
        if size <= 0. {
            return Err("The shape's size must be above 0".into());
        }
        if !(3..=1024).contains(&self.sides) {
            return Err("Sides must be 3 to 1024".into());
        }
        if !(1..=256).contains(&self.rect_segs) {
            return Err("Corner Segments must be 1 to 256".into());
        }
        if !(1..=64).contains(&self.clone_x) || !(1..=64).contains(&self.clone_y) {
            return Err("Clone X and Clone Y must be 1 to 64".into());
        }
        Ok(PrimitiveSlice {
            shape,
            place: Some(ShapePlace::on(
                DVec3::new(self.cent_x, self.cent_y, self.cent_z),
                normal,
            )),
            half_x: self.size_x,
            half_y: self.size_y,
            corner_radius: self.rect_radius.max(0.),
            corner_segments: self.rect_segs,
            radius: self.radius,
            sides: self.sides,
            heading: self.heading.to_degrees(),
            pitch: self.pitch.to_degrees(),
            bank: self.bank.to_degrees(),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            clone_x: self.clone_x,
            clone_y: self.clone_y,
            gap_x: self.clone_gap_x,
            gap_y: self.clone_gap_y,
            center_clones: self.clone_center,
            lazy_apply: false,
            ..PrimitiveSlice::default()
        })
    }
}

impl TypedOp for PrimitiveSliceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let cutter = self.cutter()?;
        let shape = self.shape()?;
        let action = shape.place.map_or(DVec3::Z, |p| p.normal);
        cutter.cut(ctx, &shape.paths(), action)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let angle = |key, label| Field::new(key, label, FieldKind::Angle);
    let mut fields = vec![
        choice("type", "Type", &SHAPES),
        distance("centX", "Center X"),
        distance("centY", "Center Y"),
        distance("centZ", "Center Z"),
        distance("vecX", "Axis X"),
        distance("vecY", "Axis Y"),
        distance("vecZ", "Axis Z"),
        distance("sizeX", "Half Size X"),
        distance("sizeY", "Half Size Y"),
        distance("radius", "Radius"),
        count("sides", "Sides", 3, 1024),
        distance("rectRadius", "Corner Radius"),
        count("rectSegs", "Corner Segments", 1, 256),
        angle("heading", "Heading"),
        angle("pitch", "Pitch"),
        angle("bank", "Bank"),
        check("flipX", "Flip X"),
        check("flipY", "Flip Y"),
        count("cloneX", "Clone X", 1, 64),
        count("cloneY", "Clone Y", 1, 64),
        distance("cloneGapX", "Clone Gap X"),
        distance("cloneGapY", "Clone Gap Y"),
        check("cloneCenter", "Center"),
    ];
    fields.extend(cutter_fields());
    vec![Arc::new(Typed::<PrimitiveSliceOp>::new(
        "prim.slice.item",
        "Primitive Slice",
        "Edit",
        true,
        fields,
    ))]
}
