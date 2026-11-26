//! Vertex map ops in the Mesh Stack (batch 4): Create Vertex Map, Set
//! Weight, Flood Weight, Set RGB, Geodesic Weight, Set Seam, Set Vertex
//! Normals.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use glam::DVec4;
use operandi_core::*;
use serde_json::json;

#[test]
fn create_vertex_map_adds_an_empty_map() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "setvmap.create.item", json!({ "MapName": "W" }));
    let m = ok(&e, id);
    assert!(m.attributes.point_weights["W"].is_empty());
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "setvmap.create.item",
        json!({ "MapName": "C", "MapType": 3 }),
    );
    assert!(ok(&e, id).attributes.point_colors.contains_key("C"));
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "setvmap.create.item", json!({}));
    assert_eq!(ok(&e, id), cube(), "a blank name adds nothing");
}

#[test]
fn set_weight_sets_and_adds() {
    let (mut e, id) = with_base(cube());
    let op = add(
        &mut e,
        id,
        "setvmap.weight.item",
        json!({ "MapValue": 0.5 }),
    );
    only(&mut e, op, "0,1");
    let m = ok(&e, id);
    let w = &m.attributes.point_weights["Weight"];
    assert_eq!(w.len(), 2);
    assert_eq!(w[&0], 0.5);
    let (mut e, id) = with_base(m);
    add(
        &mut e,
        id,
        "setvmap.weight.item",
        json!({ "MapValue": 0.25, "Additive": true }),
    );
    let m = ok(&e, id);
    let w = &m.attributes.point_weights["Weight"];
    assert_eq!(w.len(), 8);
    assert_eq!(w[&0], 0.75);
    assert_eq!(w[&5], 0.25);
}

#[test]
fn flood_weight_fills_high_or_low() {
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "setvmap.flood.item",
        json!({ "MapName": "F", "HighOrLow": 1 }),
    );
    let m = ok(&e, id);
    assert_eq!(m.attributes.point_weights["F"].len(), 8);
    assert!(m.attributes.point_weights["F"].values().all(|&w| w == 0.));
}

#[test]
fn set_rgb_colours_the_points() {
    let (mut e, id) = with_base(cube());
    let op = add(
        &mut e,
        id,
        "setvmap.rgb.item",
        json!({ "MapValue_R": 1., "MapValue_A": 0.5 }),
    );
    only(&mut e, op, "3");
    let m = ok(&e, id);
    let c = &m.attributes.point_colors["Color"];
    assert_eq!(c.len(), 1);
    assert_eq!(c[&3], DVec4::new(1., 0., 0., 1.), "RGB keeps alpha at 1");
}

#[test]
fn geodesic_weight_measures_along_edges() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "pmodel.geodesic.mOp", json!({}));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    let w = &m.attributes.point_weights["Geodesic"];
    assert_eq!(w.len(), 8);
    assert_eq!(w[&0], 0.);
    assert_eq!(w.values().cloned().fold(0., f64::max), 1.);
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "pmodel.geodesic.mOp", json!({ "invert": true }));
    only(&mut e, op, "0");
    assert_eq!(ok(&e, id).attributes.point_weights["Geodesic"][&0], 1.);
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "pmodel.geodesic.mOp", json!({}));
    assert_eq!(ok(&e, id), cube(), "no start points, no map");
}

#[test]
fn set_seam_marks_the_chosen_edges() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "setvmap.seam.item", json!({}));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    let seams = uv_seams::seam_edges(&m, uv_seams::DEFAULT_SEAM_MAP);
    assert_eq!(seams.len(), 1);
    assert!(seams.contains(&cube().edges()[0]));
}

#[test]
fn set_vertex_normals_matches_recalculate() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "setvmap.vnorm.item", json!({}));
    let want = vertex_normals::recalculate(&cube(), &Default::default()).unwrap();
    assert_eq!(ok(&e, id), want);
}
