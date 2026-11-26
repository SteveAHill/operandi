//! Deformer steps in the Mesh Stack (batch 10): Transform Effector,
//! Rotate Effector and Magnet.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use glam::{DQuat, DVec3};
use operandi_core::mesh_stack::OpStatus;
use operandi_core::*;
use serde_json::{Value, json};

fn run(id: &str, values: Value) -> Mesh {
    let (mut e, item) = with_base(cube());
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
fn a_new_transform_effector_changes_nothing() {
    assert_eq!(run("deform.transform.item", json!({})), cube());
}

#[test]
fn transform_effector_scales_turns_then_moves_about_its_center() {
    let m = run(
        "deform.transform.item",
        json!({
            "centerY": 0.5, "posX": 1., "rotZ": std::f64::consts::FRAC_PI_2,
            "sclX": 2., "strength": 1.,
        }),
    );
    let c = DVec3::new(0., 0.5, 0.);
    let turn = DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2);
    for (p, was) in m.vertices.iter().zip(&cube().vertices) {
        let want = c + turn * ((*was - c) * DVec3::new(2., 1., 1.)) + DVec3::X;
        assert!(p.distance(want) < 1e-9);
    }
}

#[test]
fn rotate_effector_turns_about_its_axis() {
    let m = run("deform.rotate.item", json!({ "axis": 0 }));
    let turn = DQuat::from_rotation_x(std::f64::consts::FRAC_PI_2);
    for (p, was) in m.vertices.iter().zip(&cube().vertices) {
        assert!(p.distance(turn * *was) < 1e-9);
    }
}

#[test]
fn magnet_pulls_only_points_within_its_radius() {
    // Centre on the top-front-right corner; Radius 1 reaches it and its
    // three neighbours (1 m away: weight 0), nothing else.
    let corner = DVec3::splat(0.5);
    let m = run(
        "deform.magnet.item",
        json!({ "centerX": 0.5, "centerY": 0.5, "centerZ": 0.5, "radius": 1. }),
    );
    for (p, was) in m.vertices.iter().zip(&cube().vertices) {
        if was.distance(corner) < 1e-9 {
            assert!(p.distance(*was + DVec3::Y * 0.5) < 1e-9, "full pull");
        } else {
            assert_eq!(p, was);
        }
    }
}

#[test]
fn effectors_reject_bad_values() {
    fails("deform.rotate.item", json!({ "axis": 3 }));
    fails("deform.magnet.item", json!({ "radius": 0. }));
}
