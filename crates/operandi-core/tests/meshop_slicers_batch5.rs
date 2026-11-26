//! Path slice ops in the Mesh Stack (batch 5): Primitive Slice, Curve
//! Slice and Pen Slice.
use crate::mesh_stack_tools::{add, cube, ok, only, quad, with_base};
use glam::DVec3;
use operandi_core::mesh_stack::OpStatus;
use operandi_core::*;
use serde_json::{Value, json};

fn polygons(m: &Mesh) -> usize {
    (0..m.faces.len()).filter(|&f| !m.is_polyline(f)).count()
}
/// Polygon `f`'s area (Newell).
fn area(m: &Mesh, f: usize) -> f64 {
    let p: Vec<DVec3> = m.faces[f]
        .vertices
        .iter()
        .map(|&v| m.vertices[v as usize])
        .collect();
    let mut n = DVec3::ZERO;
    for i in 0..p.len() {
        n += p[i].cross(p[(i + 1) % p.len()]);
    }
    n.length() / 2.
}
fn curves(m: &Mesh) -> usize {
    m.faces.len() - polygons(m)
}

/// The op fails and the mesh is its input.
fn fails(base: Mesh, id: &str, values: Value) {
    let (mut e, item) = with_base(base.clone());
    add(&mut e, item, id, values);
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap();
    assert!(matches!(r.statuses[0], OpStatus::Failed(_)), "{id}");
    assert_eq!(*r.mesh, base);
}

/// `quad()` with one open curve across it along X at z = `z` (2 points).
fn quad_and_line(z: f64) -> Mesh {
    let mut m = quad();
    let first = m.vertices.len() as u32;
    m.vertices
        .extend([DVec3::new(-2., 0.5, z), DVec3::new(2., 0.5, z)]);
    curve_commands::make_curve(&m, &[first, first + 1], false)
        .unwrap()
        .0
}

/// `quad()` with a closed 1 m square curve above its middle.
fn quad_and_square() -> Mesh {
    let mut m = quad();
    let first = m.vertices.len() as u32;
    m.vertices.extend([
        DVec3::new(-0.5, 0.5, -0.5),
        DVec3::new(0.5, 0.5, -0.5),
        DVec3::new(0.5, 0.5, 0.5),
        DVec3::new(-0.5, 0.5, 0.5),
    ]);
    curve_commands::make_curve(&m, &[first, first + 1, first + 2, first + 3], true)
        .unwrap()
        .0
}

#[test]
fn primitive_slice_cuts_a_rectangle_and_selects_its_edges() {
    let (mut e, id) = with_base(quad());
    add(
        &mut e,
        id,
        "prim.slice.item",
        json!({ "vecY": 1., "vecZ": 0., "sizeX": 0.25, "sizeY": 0.25, "selectEdges": true }),
    );
    let m = ok(&e, id);
    assert!(polygons(&m) > 1, "the quad is cut");
    // A 0.5 m square piece sits in the middle.
    let inner = (0..m.faces.len()).any(|f| (area(&m, f) - 0.25).abs() < 1e-9);
    assert!(inner);
    let set = &m.attributes.edge_sets["Sliced Edges"];
    assert!(set.len() >= 4);
}

#[test]
fn primitive_slice_circles_clone_and_stencil() {
    let (mut e, id) = with_base(quad());
    add(
        &mut e,
        id,
        "prim.slice.item",
        json!({
            "type": 1, "vecY": 1., "vecZ": 0., "radius": 0.2, "sides": 8,
            "cloneX": 2, "cloneGapX": 0.2, "cloneCenter": true,
            "mode": 2, "material": "Red",
        }),
    );
    let m = ok(&e, id);
    let red = m.faces.iter().filter(|f| f.material == "Red").count();
    assert_eq!(red, 2, "one stencilled piece per circle");
}

#[test]
fn primitive_slice_cuts_only_the_chosen_polygons() {
    let base = quad_and_line(0.);
    // Split the quad in two first, then cut a rectangle across the middle
    // into the first half only.
    let (mut e, id) = with_base(base);
    add(&mut e, id, "curve.slice.item", json!({}));
    let halves = ok(&e, id);
    assert_eq!(polygons(&halves), 2);
    let (mut e, id) = with_base(halves.clone());
    let op = add(
        &mut e,
        id,
        "prim.slice.item",
        json!({ "vecY": 1., "vecZ": 0., "sizeX": 0.3, "sizeY": 0.3 }),
    );
    let first = (0..halves.faces.len())
        .find(|&f| !halves.is_polyline(f))
        .unwrap();
    only(&mut e, op, &first.to_string());
    let m = ok(&e, id);
    assert!(polygons(&m) > 2);
    // The other half is untouched: still a 2 x 1 m polygon.
    assert!((0..m.faces.len()).any(|f| (area(&m, f) - 2.).abs() < 1e-9));
}

#[test]
fn primitive_slice_rejects_bad_values() {
    fails(quad(), "prim.slice.item", json!({ "vecZ": 0. }));
    fails(quad(), "prim.slice.item", json!({ "sizeX": 0. }));
    fails(quad(), "prim.slice.item", json!({ "type": 3 }));
    fails(quad(), "prim.slice.item", json!({ "mode": 4 }));
    fails(
        quad(),
        "prim.slice.item",
        json!({ "selectEdges": true, "selectionSetName": " " }),
    );
}

#[test]
fn curve_slice_cuts_along_a_curve_and_keeps_it() {
    let (mut e, id) = with_base(quad_and_line(0.));
    add(&mut e, id, "curve.slice.item", json!({ "number": 4 }));
    let m = ok(&e, id);
    assert_eq!(polygons(&m), 2);
    assert_eq!(curves(&m), 1, "the curve stays");
}

#[test]
fn curve_slice_start_and_end_keep_part_of_the_curve() {
    // The first fifth of the curve (x from -2 to -1.2) misses the quad.
    let (mut e, id) = with_base(quad_and_line(0.));
    add(&mut e, id, "curve.slice.item", json!({ "end": 0.2 }));
    assert_eq!(polygons(&ok(&e, id)), 1);
    // Sliding that part to the middle cuts in, without crossing: points
    // only, no split.
    let (mut e, id) = with_base(quad_and_line(0.));
    add(
        &mut e,
        id,
        "curve.slice.item",
        json!({ "end": 0.2, "slide": 0.4 }),
    );
    assert_eq!(polygons(&ok(&e, id)), 1);
    // Start 0, End 1 crosses and splits.
    let (mut e, id) = with_base(quad_and_line(0.));
    add(&mut e, id, "curve.slice.item", json!({ "auto": true }));
    assert_eq!(polygons(&ok(&e, id)), 2);
}

#[test]
fn curve_slice_with_no_curve_passes_through_and_a_bad_index_fails() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "curve.slice.item", json!({}));
    assert_eq!(ok(&e, id), cube());
    fails(quad_and_line(0.), "curve.slice.item", json!({ "index": 1 }));
    fails(quad_and_line(0.), "pen.slice.item", json!({ "index": 3 }));
}

#[test]
fn pen_slice_cuts_a_closed_curve_as_an_outline() {
    let (mut e, id) = with_base(quad_and_square());
    add(
        &mut e,
        id,
        "pen.slice.item",
        json!({ "mode": 2, "material": "Blue" }),
    );
    let m = ok(&e, id);
    assert!(polygons(&m) > 1);
    let blue: f64 = (0..m.faces.len())
        .filter(|&f| m.faces[f].material == "Blue")
        .map(|f| area(&m, f))
        .sum();
    assert!((blue - 1.).abs() < 1e-9, "the square's inside: {blue}");
    assert_eq!(curves(&m), 1);
}

#[test]
fn pen_slice_core_keeps_only_the_inside() {
    let (mut e, id) = with_base(quad_and_square());
    add(&mut e, id, "pen.slice.item", json!({ "mode": 0 }));
    let m = ok(&e, id);
    let area: f64 = (0..m.faces.len())
        .filter(|&f| !m.is_polyline(f))
        .map(|f| area(&m, f))
        .sum();
    assert!((area - 1.).abs() < 1e-9, "{area}");
}
