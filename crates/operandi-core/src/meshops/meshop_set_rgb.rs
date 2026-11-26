//! Set RGB as a Mesh Stack op (`setvmap.rgb.item`, TYPE-IDS.md): the
//! chosen points get the colour Value R, G, B (and A with Map Type RGBA)
//! in the colour map Map Name (made when missing), or, with Additive,
//! the colour added to what they have.
//!
//! Provisional (MOQ-59): a blank Map Name means the mesh's first colour
//! map ("Color" when it has none); with Map Type RGB alpha is 1. Force
//! Discontinuous, Alpha Uses Falloff and Weight Method are kept, not
//! shown or built (the op writes one colour per point).
use crate::mesh_stack_builtin::{check, choice};
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::meshop_vertex_map_common::chosen_points;
use crate::{Mesh, Result};
use glam::DVec4;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetRgbOp {
    #[serde(rename = "MapName")]
    pub map_name: String,
    #[serde(rename = "Additive")]
    pub additive: bool,
    #[serde(rename = "MapValue_R")]
    pub r: f64,
    #[serde(rename = "MapValue_G")]
    pub g: f64,
    #[serde(rename = "MapValue_B")]
    pub b: f64,
    #[serde(rename = "MapValue_A")]
    pub a: f64,
    /// 0 RGB, 1 RGBA.
    #[serde(rename = "IncludeAlpha")]
    pub include_alpha: u32,
    #[serde(rename = "AlphaUsesFalloff")]
    pub alpha_uses_falloff: bool,
    #[serde(rename = "WeightMethod")]
    pub weight_method: u32,
    #[serde(rename = "SetPerPoly")]
    pub set_per_poly: bool,
}
impl TypedOp for SetRgbOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.include_alpha > 1 {
            return Err("Map Type must be 0 (RGB) or 1 (RGBA)".into());
        }
        let alpha = if self.include_alpha == 1 { self.a } else { 1. };
        let colour = DVec4::new(self.r, self.g, self.b, alpha);
        if !colour.is_finite() {
            return Err("The colour values must be numbers".into());
        }
        let mut out = ctx.input.clone();
        let name = match self.map_name.trim() {
            "" => out
                .attributes
                .point_colors
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "Color".into()),
            n => n.to_owned(),
        };
        let map = out.attributes.point_colors.entry(name).or_default();
        for v in chosen_points(ctx) {
            let c = map.entry(v).or_insert(DVec4::ZERO);
            *c = if self.additive {
                let mut sum = *c + colour;
                if self.include_alpha == 0 {
                    sum.w = 1.;
                }
                sum
            } else {
                colour
            };
        }
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<SetRgbOp>::new(
            "setvmap.rgb.item",
            "Set RGB",
            "Vertex Map",
            true,
            vec![
                Field::new("MapName", "Map Name", FieldKind::Text),
                choice("IncludeAlpha", "Map Type", &["RGB", "RGBA"]),
                Field::new("MapValue_R", "Value R", FieldKind::Number),
                Field::new("MapValue_G", "Value G", FieldKind::Number),
                Field::new("MapValue_B", "Value B", FieldKind::Number),
                Field::new("MapValue_A", "Value A", FieldKind::Number),
                check("Additive", "Additive"),
                hidden("AlphaUsesFalloff"),
                hidden("WeightMethod"),
                hidden("SetPerPoly"),
            ],
        ))],
        crate::meshop_types::SelElement::Point,
    )
}
