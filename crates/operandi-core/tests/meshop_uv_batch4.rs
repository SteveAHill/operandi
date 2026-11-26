//! UV ops in the Mesh Stack (batch 4): Create UV Map, UV Relax, UV Unwrap
//! and Relax, UV Flip, Convert UV to Mesh.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use glam::DVec2;
use operandi_core::*;
use serde_json::json;

fn mapped(m: &Mesh, map: &str) -> Vec<DVec2> {
    let rows = m.uv_maps.get(map).expect("the map");
    assert_eq!(rows.len(), m.faces.len());
    rows.iter()
        .flatten()
        .map(|p| p.expect("every corner mapped"))
        .collect()
}

/// A cube with a planar Texture map, from the op itself.
fn mapped_cube() -> Mesh {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "uv.create.item", json!({}));
    ok(&e, id)
}

#[test]
fn create_uv_map_projects_every_polygon() {
    let m = mapped_cube();
    for p in mapped(&m, "Texture") {
        assert!((-1e-9..=1. + 1e-9).contains(&p.x) && (-1e-9..=1. + 1e-9).contains(&p.y));
    }
    for proj in 0..7 {
        let (mut e, id) = with_base(cube());
        add(
            &mut e,
            id,
            "uv.create.item",
            json!({ "proj": proj, "name": "Other" }),
        );
        mapped(&ok(&e, id), "Other");
    }
}

#[test]
fn create_uv_map_maps_only_the_chosen_polygons() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "uv.create.item", json!({}));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    let rows = &m.uv_maps["Texture"];
    assert!(rows[0].iter().all(Option::is_some));
    assert!(rows[1].iter().all(Option::is_none));
}

#[test]
fn uv_relax_runs_and_zero_iterations_leave_the_map() {
    let base = mapped_cube();
    let (mut e, id) = with_base(base.clone());
    add(&mut e, id, "uv.relax.item", json!({}));
    assert_eq!(ok(&e, id), base);
    let (mut e, id) = with_base(base);
    add(
        &mut e,
        id,
        "uv.relax.item",
        json!({ "iter": 5, "mode": 0, "boundary": 1 }),
    );
    mapped(&ok(&e, id), "Texture");
}

#[test]
fn uv_unwrap_and_relax_makes_a_map() {
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "uv.pelt.item",
        json!({ "iter": 10, "relaxiter": 5 }),
    );
    mapped(&ok(&e, id), "Texture");
}

#[test]
fn uv_flip_mirrors_about_the_middle() {
    let base = mapped_cube();
    let before = mapped(&base, "Texture");
    let (mut e, id) = with_base(base);
    add(&mut e, id, "uvflip.meshop.item", json!({}));
    let after = mapped(&ok(&e, id), "Texture");
    let (lo, hi) = before
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), p| (l.min(p.x), h.max(p.x)));
    for (b, a) in before.iter().zip(&after) {
        assert!((a.x - (lo + hi - b.x)).abs() < 1e-9);
        assert!((a.y - b.y).abs() < 1e-9);
    }
}

#[test]
fn convert_uv_to_mesh_lays_the_map_flat() {
    // Cubic, so no polygon lands edge-on in UV space.
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "uv.create.item", json!({ "proj": 6 }));
    let (mut e, id) = with_base(ok(&e, id));
    add(&mut e, id, "uvconvert.meshop.item", json!({}));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 6);
    assert!(m.vertices.iter().all(|v| v.z == 0.));
    // No such map: the mesh passes through.
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "uvconvert.meshop.item", json!({}));
    assert_eq!(ok(&e, id), cube());
}
