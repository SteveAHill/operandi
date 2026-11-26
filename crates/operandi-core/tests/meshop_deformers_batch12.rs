//! Deformer steps in the Mesh Stack (batch 12): Vortex and Lattice.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use glam::{DQuat, DVec3};
use operandi_core::mesh_stack::OpStatus;
use operandi_core::*;
use serde_json::{Value, json};

fn run(base: Mesh, id: &str, values: Value) -> Mesh {
    let (mut e, item) = with_base(base);
    add(&mut e, item, id, values);
    ok(&e, item)
}
fn fails(id: &str, values: Value) {
    let (mut e, item) = with_base(cube());
    add(&mut e, item, id, values);
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap();
    assert!(matches!(r.statuses[0], OpStatus::Failed(_)), "{id}");
}

#[test]
fn vortex_turns_most_near_its_line_and_not_past_radius() {
    // Two points: one 0.1 m from the Y axis, one 2 m away.
    let mut m = cube();
    m.vertices[0] = DVec3::new(0.1, 0., 0.);
    m.vertices[1] = DVec3::new(2., 0., 0.);
    let out = run(m.clone(), "deform.vortex.item", json!({}));
    let w = (1. - 0.01f64).powi(2);
    let turn = DQuat::from_rotation_y(std::f64::consts::FRAC_PI_2 * w);
    assert!(out.vertices[0].distance(turn * m.vertices[0]) < 1e-9);
    assert_eq!(out.vertices[1], m.vertices[1]);
}

#[test]
fn a_new_lattice_moves_nothing() {
    assert_eq!(run(cube(), "deform.lattice.item", json!({})), cube());
}

#[test]
fn moving_a_lattice_corner_moves_the_points_near_it() {
    // 2 x 2 x 2 cage on the cube: control points are the cube's corners,
    // so moving the +X+Y+Z one moves that corner of the cube by as much.
    let mut offsets = vec![[0., 0., 0.]; 8];
    offsets[7] = [0., 0.5, 0.];
    let out = run(
        cube(),
        "deform.lattice.item",
        json!({ "divX": 2, "divY": 2, "divZ": 2, "offsets": offsets }),
    );
    for (p, was) in out.vertices.iter().zip(&cube().vertices) {
        if was.min_element() > 0. {
            assert!(p.distance(*was + DVec3::Y * 0.5) < 1e-9);
        } else if was.max_element() < 0. {
            assert_eq!(p, was, "the far corner stays");
        }
    }
}

#[test]
fn lattice_leaves_points_outside_its_box_and_checks_values() {
    let mut big = cube();
    for p in &mut big.vertices {
        *p *= 4.;
    }
    let offsets = vec![[1., 1., 1.]; 27];
    let out = run(
        big.clone(),
        "deform.lattice.item",
        json!({ "offsets": offsets }),
    );
    assert_eq!(out, big);
    fails("deform.lattice.item", json!({ "divX": 1 }));
    fails("deform.lattice.item", json!({ "maxX": -1. }));
    fails("deform.vortex.item", json!({ "radius": 0. }));
}
