//! Deformer steps in the Mesh Stack (batch 13): Spline Deformer, Bezier
//! Deformer and Curve Constraint follow one of the mesh's curves.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use glam::DVec3;
use operandi_core::mesh_stack::OpStatus;
use operandi_core::*;
use serde_json::{Value, json};

/// A 0.2 x 2 x 0.2 column standing on the origin, plus one open curve
/// through `curve`.
fn column_and(curve: &[DVec3]) -> (Mesh, usize) {
    let mut m = cube();
    for p in &mut m.vertices {
        *p = DVec3::new(p.x * 0.2, p.y * 2. + 1., p.z * 0.2);
    }
    let n = m.vertices.len();
    m.vertices.extend_from_slice(curve);
    let ids: Vec<u32> = (n as u32..(n + curve.len()) as u32).collect();
    (curve_commands::make_curve(&m, &ids, false).unwrap().0, n)
}
fn run(base: Mesh, id: &str, values: Value) -> Mesh {
    let (mut e, item) = with_base(base);
    add(&mut e, item, id, values);
    ok(&e, item)
}

#[test]
fn a_straight_curve_up_the_axis_leaves_the_column() {
    let (m, _) = column_and(&[DVec3::ZERO, DVec3::Y * 3.]);
    let out = run(m.clone(), "deform.spline.item", json!({}));
    for (p, was) in out.vertices.iter().zip(&m.vertices) {
        assert!(p.distance(*was) < 1e-9, "{p} vs {was}");
    }
}

#[test]
fn the_column_follows_a_curve_along_x() {
    let (m, n) = column_and(&[DVec3::ZERO, DVec3::X * 3.]);
    let out = run(m.clone(), "deform.spline.item", json!({}));
    for (p, was) in out.vertices[..n].iter().zip(&m.vertices[..n]) {
        let want = DVec3::new(was.y, was.x, -was.z);
        assert!(p.distance(want) < 1e-9, "{p} vs {want}");
    }
    assert_eq!(out.vertices[n..], m.vertices[n..], "the curve stays");
}

#[test]
fn bezier_deformer_only_reads_bezier_curves() {
    let (m, _) = column_and(&[DVec3::ZERO, DVec3::X * 3.]);
    assert_eq!(run(m.clone(), "deform.bezier.item", json!({})), m);
}

#[test]
fn curve_constraint_pulls_onto_the_curve() {
    let (m, n) = column_and(&[DVec3::ZERO, DVec3::Y * 3.]);
    // Half way (all the way would flatten the column into a line).
    let out = run(
        m.clone(),
        "deform.curveConstraint.item",
        json!({ "strength": 0.5 }),
    );
    for (p, was) in out.vertices[..n].iter().zip(&m.vertices[..n]) {
        let onto = DVec3::new(0., was.y, 0.);
        assert!(p.distance(was.lerp(onto, 0.5)) < 1e-9);
    }
    // A Radius too small to reach: nothing moves.
    let out = run(
        m.clone(),
        "deform.curveConstraint.item",
        json!({ "radius": 0.05 }),
    );
    assert_eq!(out, m);
}

#[test]
fn curve_deformers_pass_through_without_curves_and_check_the_index() {
    assert_eq!(run(cube(), "deform.spline.item", json!({})), cube());
    let (m, _) = column_and(&[DVec3::ZERO, DVec3::Y]);
    let (mut e, item) = with_base(m);
    add(&mut e, item, "deform.spline.item", json!({ "index": 2 }));
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap();
    assert!(matches!(r.statuses[0], OpStatus::Failed(_)));
}
