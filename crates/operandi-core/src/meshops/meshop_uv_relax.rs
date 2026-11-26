//! UV Relax as a Mesh Stack op (`uv.relax.item`, TYPE-IDS.md): the UV
//! Relax tool (`uv_relax::relax_mesh`, UVS-1 §3) on the chosen polygons'
//! UVs in the map Name: Mode, Iterations, Seal Holes, Boundary, Area
//! Weight and Uniform Spans. Iterations starts at 0 (the reference's
//! default), which leaves the UVs as they are.
//!
//! Provisional (MOQ-53): Mode is stored by position (Spring, Conformal =
//! the reference's "lscm", Angle Based, Adaptive) and Boundary likewise;
//! the transform handles' channels (offset, scale, pivot, angle) and the
//! interactive ones are kept, not shown or built. Polygons the op didn't
//! choose stay still.
use crate::mesh_stack_builtin::{check, choice, count};
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::uv_relax::{RelaxBoundary, RelaxMode, RelaxOptions};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UvRelaxOp {
    pub mode: u32,
    pub iter: u32,
    pub live: bool,
    pub seal: bool,
    pub boundary: u32,
    pub area: f64,
    pub pick: bool,
    pub uniform: bool,
    pub pause: bool,
    pub no_edge: bool,
    pub name: String,
    pub show_handle: bool,
    pub offset_u: f64,
    pub offset_v: f64,
    pub scale_u: f64,
    pub scale_v: f64,
    pub pivot_u: f64,
    pub pivot_v: f64,
    pub angle: f64,
}
impl Default for UvRelaxOp {
    fn default() -> Self {
        Self {
            mode: 1,
            iter: 0,
            live: false,
            seal: false,
            boundary: 0,
            area: 0.1,
            pick: false,
            uniform: false,
            pause: false,
            no_edge: true,
            name: String::new(),
            show_handle: false,
            offset_u: 0.,
            offset_v: 0.,
            scale_u: 1.,
            scale_v: 1.,
            pivot_u: 0.5,
            pivot_v: 0.5,
            angle: 0.,
        }
    }
}

/// The Relax options for a mode, boundary and the rest, by position.
pub(crate) fn relax_options(
    mode: u32,
    iterations: u32,
    seal: bool,
    boundary: u32,
    area: f64,
    uniform: bool,
    map: String,
) -> Result<RelaxOptions> {
    Ok(RelaxOptions {
        uv_map: Some(map),
        mode: *RelaxMode::ALL
            .get(mode as usize)
            .ok_or("Mode must be 0 (Spring) to 3 (Adaptive)")?,
        iterations,
        seal_holes: seal,
        boundary: *RelaxBoundary::ALL
            .get(boundary as usize)
            .ok_or("Boundary must be 0 (Smooth) to 3 (Straighten)")?,
        uniform_spans: uniform,
        area_weight: area,
        ..Default::default()
    })
}

/// Relax `input`'s map `o.uv_map` on the op's chosen polygons.
pub(crate) fn relax_chosen(ctx: &OpContext, input: &Mesh, o: &RelaxOptions) -> Result<Mesh> {
    if o.iterations == 0 {
        return Ok(input.clone());
    }
    let map = o.uv_map.clone().unwrap_or_default();
    if !input.uv_maps.contains_key(&map) {
        return Err(format!("The mesh has no UV map called {map}"));
    }
    let (scene, _) = crate::meshop_scene::one_mesh_scene(ctx, input, true)?;
    crate::uv_relax::relax_mesh(&scene, o, None)
}

impl TypedOp for UvRelaxOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let name = self.name.trim();
        let map = crate::uv_tools::target_uv_map(ctx.input, (!name.is_empty()).then_some(name));
        let o = relax_options(
            self.mode,
            self.iter,
            self.seal,
            self.boundary,
            self.area,
            self.uniform,
            map,
        )?;
        relax_chosen(ctx, ctx.input, &o)
    }
}

pub(crate) const MODES: [&str; 4] = ["Spring", "Conformal", "Angle Based", "Adaptive"];
pub(crate) const BOUNDARIES: [&str; 4] = ["Smooth", "Locked", "Maintain", "Straighten"];

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<UvRelaxOp>::new(
        "uv.relax.item",
        "UV Relax",
        "UV",
        true,
        vec![
            Field::new("name", "UV Map", FieldKind::Text),
            choice("mode", "Mode", &MODES),
            count("iter", "Iterations", 0, 100_000),
            check("seal", "Seal Holes"),
            choice("boundary", "Boundary", &BOUNDARIES),
            Field::new("area", "Area Weight", FieldKind::Percent),
            check("uniform", "Uniform Spans"),
            hidden("live"),
            hidden("pick"),
            hidden("pause"),
            hidden("noEdge"),
            hidden("showHandle"),
            hidden("offsetU"),
            hidden("offsetV"),
            hidden("scaleU"),
            hidden("scaleV"),
            hidden("pivotU"),
            hidden("pivotV"),
            hidden("angle"),
        ],
    ))]
}
