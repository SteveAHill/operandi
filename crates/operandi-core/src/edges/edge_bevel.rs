//! ME-T01 Edge Bevel and ME-T07 Edge Chamfer.
//!
//! Both widen selected edges into strips while the
//! polygons on either side stay in their planes and shrink:
//! * **Edge Bevel** (Inset mode): each end of the edge moves the Inset
//!   distance along the side edges of the neighbouring polygons. Round Level
//!   adds rows between the two new edges.
//! * **Edge Chamfer**: the new edges are offset in parallel to the source edge
//!   (each end moves along its side edge far enough that the new edge is
//!   Offset away), which keeps the strip even. Segments rows round it.
//!
//! Rows follow a circular arc whose corner is the source edge (Depth 100%);
//! Depth 0% keeps them flat on the chord. The construction itself is in
//! `edge_bevel_junctions` (specifications/edge-bevel-chamfer-2026-11-02);
//! this module holds the options and the entry points. UV maps follow the
//! shared provisional rule in `corner_uv`; meshes with other per-corner data
//! are refused for now.
use crate::topology::edge_key;
use crate::{Mesh, Result};
use std::collections::{BTreeMap, BTreeSet};

/// How the width is measured (EBC-D-01/02, EBC-P-04, EBC-B3-01).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeBevelMode {
    /// Each side moves the width along its polygon.
    #[default]
    Inset,
    /// The width is read across the corner the edge makes; the distance each
    /// side moves depends on the angle between the two polygons.
    Width,
}

/// Profile shape (EBC-D-01: Round, Square, Sharp; Edge Chamfer has Round and
/// Square).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeBevelShape {
    #[default]
    Round,
    /// EBC-P-02 / EBC-B3-08: the profile goes through the original edge, two
    /// flat segments whatever the level.
    Square,
    /// Measured the same as Square at Levels 1–3 (EBC-B3-08).
    Sharp,
}

/// Edge Chamfer's Edge End (EBC-D-02, EBC-B3-07).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeEnd {
    #[default]
    Profile,
    TriangleFan,
}

/// Edge Bevel's Mitering Style (a tester's request, 2026-11-24, named
/// Corner Pattern until 2026-11-25; proposal, EBC-Q-CP1): how the opening
/// is filled where strips of two or more rows meet at a point. The panel
/// shows `NoMitering` as Neutral and `Balanced` as Default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CornerPattern {
    /// Balanced's layout with nothing bent sideways (the tester's Maya No
    /// Mitering picture, 2026-11-24; shown as Neutral): the
    /// strips that end on the rounded outline end in a round standing
    /// upright, so the corner's lines run straight up to the outline and
    /// its rows are plain chevrons.
    NoMitering,
    /// Where two split lines run into the corner, the outline between them
    /// is rounded around the old point and the corner is an even grid, the
    /// rows wrapping across (the tester's pick, 2026-11-24, formerly None).
    /// Elsewhere, Patch's grid. Shown as Default, and the default (the
    /// tester, 2026-11-26).
    #[default]
    Balanced,
    /// EBC-J-05: a grid of quads per strip corner, each strip's rows bending
    /// around into the next (the measured fill).
    Patch,
    /// The third strip's lines close in on one pole like a pointed (gothic)
    /// arch, and the rows nest under it as smooth arches shaped like that
    /// strip's end (Maya's Uniform; smoothed for the tester, 2026-11-25).
    Uniform,
    /// A narrow arch pinched to a sharp spike, its rows meeting in straight
    /// V seams (the tester's request, 2026-11-24, from an earlier build).
    Pinch,
    /// Rings around one pole point, one per row pair, closed at the pole.
    Radial,
}

impl CornerPattern {
    /// Balanced and No Mitering: the outline between two split lines is
    /// rounded instead of closed to one mitre point.
    pub fn rounds_outline(self) -> bool {
        matches!(self, Self::Balanced | Self::NoMitering)
    }
}

/// The options of Edge Bevel and Edge Chamfer besides width, rows and Depth.
/// Defaults are the values after Reset (EBC-D-01/02).
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeBevelOptions {
    pub mode: EdgeBevelMode,
    pub shape: EdgeBevelShape,
    /// Edge Chamfer: each side moves far enough that the new edge is Offset
    /// away from the source edge (on), or the Offset along the side edge
    /// (off, like Edge Bevel's Inset). Edge Bevel ignores it.
    pub parallel: bool,
    /// A side point that would pass the far end of its edge stops there
    /// (on), or carries on along the same line (off). Provisional reading
    /// (EBC-Q10).
    pub stop_at_edge: bool,
    /// New points that land on another point merge with it (provisional,
    /// EBC-Q10).
    pub auto_weld: bool,
    pub edge_end: EdgeEnd,
    /// The new polygons get `material` (provisional, EBC-Q05).
    pub use_material: bool,
    pub material: String,
    /// Edge Bevel's Mitering field and second handle (a tester's request,
    /// 2026-11-23; called Support Edges in this module): one more row of
    /// edges this far beyond the bevel on each side, in the neighbouring
    /// polygons' planes; 0 adds none. Edge Chamfer ignores it.
    pub support: f64,
    /// Edge Bevel's Loop Slide (a tester's request, 2026-11-24; on by
    /// default). Where an unbevelled edge runs between two bevelled edges
    /// into a point (an inset's diagonal between two rim edges), on: the
    /// new point slides along that edge, so it stays straight and the edge
    /// flow is kept. Off: the new point goes where the two strips' sides
    /// meet, an even width from both, and that edge bends to it. Edge
    /// Chamfer ignores it.
    pub loop_slide: bool,
    /// Edge Bevel: how corners where strips meet are filled (Patch, the
    /// measured fill, by default). Edge Chamfer ignores it.
    pub corner: CornerPattern,
}
impl Default for EdgeBevelOptions {
    fn default() -> Self {
        Self {
            mode: EdgeBevelMode::Inset,
            shape: EdgeBevelShape::Round,
            parallel: true,
            stop_at_edge: true,
            auto_weld: false,
            edge_end: EdgeEnd::Profile,
            use_material: false,
            material: "Default".into(),
            support: 0.,
            loop_slide: true,
            corner: CornerPattern::Balanced,
        }
    }
}

/// What Edge Bevel / Edge Chamfer keep between uses: everything but the
/// width, which starts at zero each time (provisional, EBC-Q10).
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeBevelPreset {
    pub segments: u32,
    pub depth: f64,
    pub options: EdgeBevelOptions,
}
impl Default for EdgeBevelPreset {
    fn default() -> Self {
        Self {
            segments: 1,
            depth: 100.,
            options: Default::default(),
        }
    }
}

/// One Edge Bevel / Edge Chamfer application.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeBevelSettings {
    /// Inset (Edge Bevel), Offset (Edge Chamfer) or Width, in mesh units.
    pub width: f64,
    /// Profile segments: Segments, or 2 × Round Level (EBC-B3-02), at least 1.
    pub rows: u32,
    /// Depth in percent (EBC-P-03).
    pub depth: f64,
    /// Edge Chamfer rather than Edge Bevel.
    pub chamfer: bool,
    pub options: EdgeBevelOptions,
}

/// The beveled mesh and what each tool selects afterwards.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeBevelResult {
    pub mesh: Mesh,
    /// Every edge of the new strips.
    pub strip_edges: Vec<(u32, u32)>,
    /// What the tool selects after it applies: the strips' edges for Edge
    /// Bevel (EBC-J-01), nothing for Edge Chamfer, except the edges through
    /// the original corner with the Square shape (EBC-P-02).
    pub selected_after: Vec<(u32, u32)>,
}

/// Bevel (`parallel = false`) or chamfer (`parallel = true`) the selected
/// edges by `width` with `rows` rows (1 = flat strip) and `depth` (0..100 %).
/// Returns the mesh and the strip's edges.
pub fn edge_bevel_geometry(
    mesh: &Mesh,
    edges: &[(u32, u32)],
    width: f64,
    rows: u32,
    depth: f64,
    parallel: bool,
) -> Result<(Mesh, Vec<(u32, u32)>)> {
    edge_bevel_geometry_weighted(mesh, edges, width, rows, depth, parallel, None)
}

/// Edge Bevel with a falloff: the width at each end of an edge is scaled by
/// that end point's weight (FAL-T-02).
pub fn edge_bevel_geometry_weighted(
    mesh: &Mesh,
    edges: &[(u32, u32)],
    width: f64,
    rows: u32,
    depth: f64,
    parallel: bool,
    weights: Option<&[f64]>,
) -> Result<(Mesh, Vec<(u32, u32)>)> {
    let settings = EdgeBevelSettings {
        width,
        rows,
        depth,
        chamfer: parallel,
        options: Default::default(),
    };
    edge_bevel_with(mesh, edges, &settings, weights).map(|r| (r.mesh, r.strip_edges))
}

/// Edge Bevel / Edge Chamfer with every option.
pub fn edge_bevel_with(
    mesh: &Mesh,
    edges: &[(u32, u32)],
    settings: &EdgeBevelSettings,
    weights: Option<&[f64]>,
) -> Result<EdgeBevelResult> {
    let what = if settings.chamfer {
        "Edge Chamfer"
    } else {
        "Edge Bevel"
    };
    mesh.validate()?;
    let EdgeBevelSettings {
        width, rows, depth, ..
    } = *settings;
    if !width.is_finite()
        || width < 0.
        || !(1..=64).contains(&rows)
        || !(0. ..=100.).contains(&depth)
    {
        return Err(format!(
            "{what} needs a nonnegative width, 1–64 rows and Depth 0–100%"
        ));
    }
    if settings.options.use_material && settings.options.material.trim().is_empty() {
        return Err(format!(
            "{what}: enter a material name or untick Use Material"
        ));
    }
    let selected: BTreeSet<(u32, u32)> = edges.iter().map(|&(a, b)| edge_key(a, b)).collect();
    if selected.is_empty() {
        return Err(format!("Select edges for {what}"));
    }
    let support = settings.options.support;
    if !support.is_finite() || support < 0. {
        return Err(format!(
            "{what}: Mitering Offset must be a distance of 0 or more"
        ));
    }
    if width == 0. && support > 0. && !settings.chamfer {
        // No bevel, only the inset: an edge on each side of each selected
        // edge, `support` away in its polygons' planes, the edge itself kept
        // (the Square shape does exactly this, EBC-P-02).
        let square = EdgeBevelSettings {
            width: support,
            rows: 1,
            options: EdgeBevelOptions {
                shape: EdgeBevelShape::Square,
                support: 0.,
                ..settings.options.clone()
            },
            ..settings.clone()
        };
        return edge_bevel_with(mesh, edges, &square, weights);
    }
    if width == 0. {
        let edges: Vec<_> = selected.into_iter().collect();
        return Ok(EdgeBevelResult {
            mesh: mesh.clone(),
            selected_after: if settings.chamfer {
                Vec::new()
            } else {
                edges.clone()
            },
            strip_edges: edges,
        });
    }
    // Vertex normal maps are carried (`edge_bevel_junctions`): a mesh with
    // hard edges always has one, and refusing it left Edge Bevel doing
    // nothing on such meshes (forum report, 2026-11-22).
    if crate::corner_uv::has_uncarried_corner_data_besides_normals(mesh)
        || !mesh.attributes.polygon_sets.is_empty()
        || !mesh.attributes.extrude_edge_uses.is_empty()
    {
        return Err(format!(
            "{what} cannot yet remap corner weights, colors or polygon sets; nothing changed"
        ));
    }
    if weights.is_some_and(|w| w.len() != mesh.vertices.len()) {
        return Err(format!("{what}: falloff weights are out of date"));
    }
    if settings.chamfer || support == 0. {
        return crate::edge_bevel_junctions::general(mesh, &selected, settings, weights, what);
    }
    let plain = EdgeBevelSettings {
        options: EdgeBevelOptions {
            support: 0.,
            ..settings.options.clone()
        },
        ..settings.clone()
    };
    let bevelled = crate::edge_bevel_junctions::general(mesh, &selected, &plain, weights, what)?;
    // The polygons beside the bevelled edges get the rows (the bevel keeps
    // their slots); the polygons at the strips' ends don't.
    let beside: BTreeSet<usize> = (0..mesh.faces.len())
        .filter(|&f| {
            let v = &mesh.faces[f].vertices;
            (0..v.len()).any(|i| selected.contains(&edge_key(v[i], v[(i + 1) % v.len()])))
        })
        .collect();
    add_support_edges(bevelled, &beside, support, what)
}

/// Support Edges on a finished bevel: in each polygon `beside` a bevelled
/// edge, next to each side of a strip (a strip edge on it), a row of
/// edges `support` further along that polygon's side edges, cut through
/// the shared slice engine so every polygon on those side edges takes the
/// new points and corner data follows. A side edge too short for the
/// distance gets no row there.
fn add_support_edges(
    bevel: EdgeBevelResult,
    beside: &BTreeSet<usize>,
    support: f64,
    what: &str,
) -> Result<EdgeBevelResult> {
    use crate::slice_cut::{Chord, EdgeCut, InnerPoint, Node};
    let mesh = &bevel.mesh;
    let strip: BTreeSet<(u32, u32)> = bevel
        .strip_edges
        .iter()
        .map(|&(a, b)| edge_key(a, b))
        .collect();
    let ring = |f: usize| {
        let v = &mesh.faces[f].vertices;
        (0..v.len()).map(move |i| (v[i], v[(i + 1) % v.len()]))
    };
    // A strip polygon: every one of its edges is a strip edge.
    let is_strip: Vec<bool> = (0..mesh.faces.len())
        .map(|f| !mesh.is_polyline(f) && ring(f).all(|(a, b)| strip.contains(&edge_key(a, b))))
        .collect();
    let mut on_strip_polygon: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in (0..mesh.faces.len()).filter(|&f| is_strip[f]) {
        on_strip_polygon.extend(ring(f).map(|(a, b)| edge_key(a, b)));
    }
    let at = |v: u32| mesh.vertices[v as usize];
    let unit = |from: u32, to: u32| (at(to) - at(from)).normalize_or_zero();
    let mut cuts: Vec<EdgeCut> = Vec::new();
    let mut inner: Vec<InnerPoint> = Vec::new();
    let mut chords: Vec<Chord> = Vec::new();
    for (f, &strip_polygon) in is_strip.iter().enumerate() {
        if strip_polygon || mesh.is_polyline(f) || !beside.contains(&f) {
            continue;
        }
        let v = &mesh.faces[f].vertices;
        let n = v.len();
        // Which of the polygon's sides lie on a strip (side i runs from
        // corner i to corner i + 1).
        let on: Vec<bool> = (0..n)
            .map(|i| on_strip_polygon.contains(&edge_key(v[i], v[(i + 1) % n])))
            .collect();
        if on.iter().all(|&o| o) {
            continue;
        }
        // Each run of strip sides gets one row: it starts and ends on the
        // side edges leaving the run, and turns each corner inside the run at
        // `support` from both of its strip sides (an inset corner).
        for first in (0..n).filter(|&i| on[i] && !on[(i + n - 1) % n]) {
            let mut last = first;
            while on[(last + 1) % n] {
                last = (last + 1) % n;
            }
            let start = v[first];
            let end = v[(last + 1) % n];
            let (from_side, to_side) = (v[(first + n - 1) % n], v[(last + 2) % n]);
            let along = |from: u32, to: u32| {
                let length = at(from).distance(at(to));
                (length > support * (1. + 1e-9)).then(|| support / length)
            };
            let (Some(t0), Some(t1)) = (along(start, from_side), along(end, to_side)) else {
                continue;
            };
            let mut through = Vec::new();
            let mut i = first;
            while i != last {
                let corner = (i + 1) % n;
                let (c, back, ahead) = (v[corner], v[i], v[(corner + 1) % n]);
                inner.push(InnerPoint {
                    face: f,
                    at: at(c) + (unit(c, back) + unit(c, ahead)) * support,
                });
                through.push(inner.len() - 1);
                i = corner;
            }
            cuts.push(EdgeCut {
                a: start,
                b: from_side,
                t: t0,
                at: None,
            });
            cuts.push(EdgeCut {
                a: end,
                b: to_side,
                t: t1,
                at: None,
            });
            chords.push(Chord {
                face: f,
                from: Node::Cut(cuts.len() - 2),
                to: Node::Cut(cuts.len() - 1),
                through,
            });
        }
    }
    if chords.is_empty() {
        return Ok(bevel);
    }
    let cut = crate::slice_cut::cut(mesh, &cuts, &inner, &chords)
        .map_err(|e| format!("{what}: Mitering Offset: {e}"))?;
    // The rows split some of the bevel's own edges (where a strip's end
    // lies along a side edge, as at the open ends of three sides of a
    // square). The strips and the selection name the pieces instead: the
    // old edges no longer exist, and a selection naming them was refused
    // on the next change ("Invalid component selection").
    let mut on_edge: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (c, &point) in cuts.iter().zip(&cut.cut_points) {
        let points = on_edge.entry(edge_key(c.a, c.b)).or_default();
        if !points.contains(&point) {
            points.push(point);
        }
    }
    let split = |edges: &[(u32, u32)]| -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        for &(a, b) in edges {
            let Some(points) = on_edge.get(&edge_key(a, b)) else {
                out.push((a, b));
                continue;
            };
            let from = cut.mesh.vertices[a as usize];
            let mut chain = points.clone();
            chain.sort_by(|&p, &q| {
                let d = |v: u32| cut.mesh.vertices[v as usize].distance_squared(from);
                d(p).total_cmp(&d(q))
            });
            chain.insert(0, a);
            chain.push(b);
            out.extend(chain.windows(2).map(|w| edge_key(w[0], w[1])));
        }
        out.sort_unstable();
        out.dedup();
        out
    };
    Ok(EdgeBevelResult {
        strip_edges: split(&bevel.strip_edges),
        selected_after: split(&bevel.selected_after),
        mesh: cut.mesh,
    })
}

/// EdgeBevelTool's last-used values (preset_store.rs).
pub enum EdgeBevelToolPreset {}
impl crate::PresetKey for EdgeBevelToolPreset {
    type Value = EdgeBevelPreset;
}

/// EdgeChamferTool's last-used values (preset_store.rs).
pub enum EdgeChamferToolPreset {}
impl crate::PresetKey for EdgeChamferToolPreset {
    type Value = EdgeBevelPreset;
}
