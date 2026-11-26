//! UV Unwrap and Relax as a Mesh Stack op (`uv.pelt.item`, TYPE-IDS.md):
//! UV Unwrap's step (when its Enable is on) followed by UV Relax's (when
//! its Enable is on), on the chosen polygons, with each step's own
//! channels. Both Iterations start at 0, as in the reference.
//!
//! Provisional (MOQ-54): the unwrap channels UV Unwrap's op doesn't build
//! yet (MOQ-26) are kept, not shown; Relax Mode starts at Adaptive and,
//! like Boundary, is stored by position (MOQ-53).
use crate::mesh_stack_builtin::{check, choice, count};
use crate::mesh_stack_tools::{UvUnwrapOp, hidden};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::meshop_uv_relax::{BOUNDARIES, MODES, relax_chosen, relax_options};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UvUnwrapRelaxOp {
    #[serde(rename = "enableU")]
    pub enable_u: bool,
    pub iter: u32,
    pub gaps: f64,
    pub matr: bool,
    pub part: bool,
    pub project: u32,
    pub pinning: u32,
    pub axis: u32,
    pub angle: f64,
    pub seal: bool,
    pub layout: bool,
    pub mode: u32,
    pub symmetrize: bool,
    pub symm_axis: u32,
    pub symm_comp: u32,
    pub name: String,
    pub seam: u32,
    pub seam_name: String,
    pub pick_name: String,
    #[serde(rename = "enableR")]
    pub enable_r: bool,
    pub relaxmode: u32,
    pub relaxiter: u32,
    pub boundary: u32,
    pub area: f64,
    pub uniform: bool,
}
impl Default for UvUnwrapRelaxOp {
    fn default() -> Self {
        let u = UvUnwrapOp::default();
        Self {
            enable_u: true,
            iter: 0,
            gaps: u.gaps,
            matr: u.matr,
            part: u.part,
            project: u.project,
            pinning: u.pinning,
            axis: u.axis,
            angle: u.angle,
            seal: u.seal,
            layout: u.layout,
            mode: u.mode,
            symmetrize: u.symmetrize,
            symm_axis: u.symm_axis,
            symm_comp: u.symm_comp,
            name: u.name,
            seam: u.seam,
            seam_name: u.seam_name,
            pick_name: u.pick_name,
            enable_r: true,
            relaxmode: 3,
            relaxiter: 0,
            boundary: 0,
            area: 0.1,
            uniform: false,
        }
    }
}
impl TypedOp for UvUnwrapRelaxOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let unwrapped = if self.enable_u {
            let u = UvUnwrapOp {
                iter: self.iter,
                gaps: self.gaps,
                matr: self.matr,
                part: self.part,
                project: self.project,
                pinning: self.pinning,
                axis: self.axis,
                angle: self.angle,
                seal: self.seal,
                layout: self.layout,
                mode: self.mode,
                symmetrize: self.symmetrize,
                symm_axis: self.symm_axis,
                symm_comp: self.symm_comp,
                name: self.name.clone(),
                seam: self.seam,
                seam_name: self.seam_name.clone(),
                pick_name: self.pick_name.clone(),
            };
            u.run(ctx)?
        } else {
            ctx.input.clone()
        };
        if !self.enable_r || self.relaxiter == 0 {
            return Ok(unwrapped);
        }
        let name = self.name.trim();
        let map = crate::uv_tools::target_uv_map(&unwrapped, (!name.is_empty()).then_some(name));
        let o = relax_options(
            self.relaxmode,
            self.relaxiter,
            self.seal,
            self.boundary,
            self.area,
            self.uniform,
            map,
        )?;
        relax_chosen(ctx, &unwrapped, &o)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<UvUnwrapRelaxOp>::new(
        "uv.pelt.item",
        "UV Unwrap and Relax",
        "UV",
        true,
        vec![
            Field::new("name", "UV Map", FieldKind::Text),
            check("enableU", "Unwrap"),
            count("iter", "Unwrap Iterations", 0, 100_000),
            Field::new("gaps", "Relative Gap Size", FieldKind::Percent),
            check("matr", "Segment by Material"),
            check("part", "Segment by Part"),
            Field::new("angle", "Offset Angle", FieldKind::Angle),
            check("seal", "Seal Holes"),
            check("layout", "Update Layout"),
            check("enableR", "Relax"),
            choice("relaxmode", "Relax Mode", &MODES),
            count("relaxiter", "Relax Iterations", 0, 100_000),
            choice("boundary", "Boundary", &BOUNDARIES),
            Field::new("area", "Area Weight", FieldKind::Percent),
            check("uniform", "Uniform Spans"),
            hidden("project"),
            hidden("pinning"),
            hidden("axis"),
            hidden("mode"),
            hidden("symmetrize"),
            hidden("symmAxis"),
            hidden("symmComp"),
            hidden("seam"),
            hidden("seamName"),
            hidden("pickName"),
        ],
    ))]
}
