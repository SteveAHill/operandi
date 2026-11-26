//! Deformer steps in the Mesh Stack (batch 8): Bend and Twist move points
//! only, by Strength, on the Selection's points.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use glam::{DQuat, DVec3};
use operandi_core::mesh_stack::OpStatus;
use operandi_core::*;
use serde_json::{Value, json};

/// A 0.2 x 2 x 0.2 column standing on the origin.
fn column() -> Mesh {
    let mut m = cube();
    for p in &mut m.vertices {
        *p = DVec3::new(p.x * 0.2, p.y * 2. + 1., p.z * 0.2);
    }
    m
}

fn fails(base: Mesh, id: &str, values: Value) {
    let (mut e, item) = with_base(base.clone());
    add(&mut e, item, id, values);
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap();
    assert!(matches!(r.statuses[0], OpStatus::Failed(_)), "{id}");
}

#[test]
fn bend_curls_the_spine_and_keeps_the_points() {
    let c = column();
    let (mut e, id) = with_base(c.clone());
    add(
        &mut e,
        id,
        "deform.bend.item",
        json!({ "spineY": 2., "axis": 2 }),
    );
    let m = ok(&e, id);
    assert_eq!(m.faces, c.faces, "topology stays");
    // The base stays; the top turns 90° about Z, so it ends up level.
    for (p, was) in m.vertices.iter().zip(&c.vertices) {
        if was.y < 1e-9 {
            assert!(p.distance(*was) < 1e-9);
        } else {
            assert!(p.y < 2. - 1e-3, "the top comes down: {p}");
        }
    }
}

#[test]
fn twist_turns_by_distance_along_the_axis() {
    let c = column();
    let (mut e, id) = with_base(c.clone());
    add(&mut e, id, "deform.twist.item", json!({ "length": 2. }));
    let m = ok(&e, id);
    let turn = DQuat::from_axis_angle(DVec3::Y, std::f64::consts::FRAC_PI_2);
    for (p, was) in m.vertices.iter().zip(&c.vertices) {
        let want = if was.y > 1. { turn * *was } else { *was };
        assert!(p.distance(want) < 1e-9, "{p} vs {want}");
    }
}

#[test]
fn strength_and_selection_limit_the_move() {
    let c = column();
    let full = {
        let (mut e, id) = with_base(c.clone());
        add(&mut e, id, "deform.twist.item", json!({ "length": 2. }));
        ok(&e, id)
    };
    let (mut e, id) = with_base(c.clone());
    add(
        &mut e,
        id,
        "deform.twist.item",
        json!({ "length": 2., "strength": 0.5 }),
    );
    let half = ok(&e, id);
    for ((h, f), was) in half.vertices.iter().zip(&full.vertices).zip(&c.vertices) {
        assert!(h.distance(was.lerp(*f, 0.5)) < 1e-9);
    }
    // Only one top point picked: only it moves.
    let top = c.vertices.iter().position(|p| p.y > 1.).unwrap();
    let (mut e, id) = with_base(c.clone());
    let op = add(&mut e, id, "deform.twist.item", json!({ "length": 2. }));
    only(&mut e, op, &top.to_string());
    let m = ok(&e, id);
    for (i, (p, was)) in m.vertices.iter().zip(&c.vertices).enumerate() {
        if i == top {
            assert!(p.distance(full.vertices[i]) < 1e-9);
        } else {
            assert_eq!(p, was);
        }
    }
}

#[test]
fn deformers_reject_bad_values() {
    fails(column(), "deform.bend.item", json!({ "axis": 3 }));
    fails(
        column(),
        "deform.bend.item",
        json!({ "spineX": 0., "spineY": 0., "spineZ": 0. }),
    );
    fails(column(), "deform.twist.item", json!({ "length": 0. }));
    fails(column(), "deform.twist.item", json!({ "axis": 5 }));
}
