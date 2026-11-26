//! Tool pipe steps in the Mesh Stack (batch 6): Symmetry and Set Work
//! Plane change the steps above them.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use glam::{DMat4, DQuat, DVec3, EulerRot};
use operandi_core::mesh_stack_editor::OpRef;
use operandi_core::*;
use serde_json::json;

/// The cube's polygon facing `n`.
fn facing(m: &Mesh, n: DVec3) -> usize {
    (0..m.faces.len())
        .find(|&f| m.normal(f).dot(n) > 0.99)
        .unwrap()
}
fn red(m: &Mesh) -> Vec<usize> {
    (0..m.faces.len())
        .filter(|&f| m.faces[f].material == "Red")
        .collect()
}
fn tag_plus_x(e: &mut Editor, id: u64) -> OpRef {
    let op = add(
        e,
        id,
        "pmodel.materialTag.item",
        json!({ "materialName": "Red" }),
    );
    let f = facing(&cube(), DVec3::X);
    only(e, op, &f.to_string());
    op
}

#[test]
fn symmetry_mirrors_the_selection_of_the_steps_above() {
    let c = cube();
    let (mut e, id) = with_base(c.clone());
    add(&mut e, id, "symmetry.tool.item", json!({ "axis": 0 }));
    tag_plus_x(&mut e, id);
    let m = ok(&e, id);
    let mut want = vec![facing(&c, DVec3::X), facing(&c, -DVec3::X)];
    want.sort_unstable();
    assert_eq!(red(&m), want);
    // Across Y the +X face's partner is itself.
    let (mut e, id) = with_base(c.clone());
    add(&mut e, id, "symmetry.tool.item", json!({ "axis": 1 }));
    tag_plus_x(&mut e, id);
    assert_eq!(red(&ok(&e, id)), vec![facing(&c, DVec3::X)]);
}

#[test]
fn symmetry_only_reaches_up_and_a_turned_off_step_sets_nothing() {
    let c = cube();
    // Below the Symmetry step: not mirrored.
    let (mut e, id) = with_base(c.clone());
    tag_plus_x(&mut e, id);
    add(&mut e, id, "symmetry.tool.item", json!({}));
    assert_eq!(red(&ok(&e, id)), vec![facing(&c, DVec3::X)]);
    // Turned off.
    let (mut e, id) = with_base(c.clone());
    let s = add(&mut e, id, "symmetry.tool.item", json!({}));
    tag_plus_x(&mut e, id);
    e.dispatch(Command::MeshOpEnable { op: s, on: false })
        .unwrap();
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert_eq!(red(&r.mesh), vec![facing(&c, DVec3::X)]);
}

#[test]
fn symmetry_mirrors_points() {
    let c = cube();
    let (mut e, id) = with_base(c.clone());
    add(&mut e, id, "symmetry.tool.item", json!({}));
    let op = add(&mut e, id, "setvmap.flood.item", json!({}));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    let p = c.vertices[0];
    let mirror = c
        .vertices
        .iter()
        .position(|q| q.distance(DVec3::new(-p.x, p.y, p.z)) < 1e-9)
        .unwrap();
    let w = m
        .attributes
        .point_weights
        .values()
        .next()
        .expect("a weight map");
    let full: Vec<usize> = w
        .iter()
        .filter(|(_, v)| **v > 0.5)
        .map(|(i, _)| *i as usize)
        .collect();
    let mut want = vec![0, mirror];
    want.sort_unstable();
    assert_eq!(full, want);
}

#[test]
fn set_work_plane_places_new_primitives_and_leaves_the_rest() {
    let plain = {
        let (mut e, id) = with_base(cube());
        add(&mut e, id, "prim.cube.item", json!({}));
        ok(&e, id)
    };
    let values = json!({ "offsety": 2., "anglez": std::f64::consts::FRAC_PI_2 });
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "pmodel.workplane.item", values);
    add(&mut e, id, "prim.cube.item", json!({}));
    let m = ok(&e, id);
    let frame = DMat4::from_rotation_translation(
        DQuat::from_euler(EulerRot::ZYX, std::f64::consts::FRAC_PI_2, 0., 0.),
        DVec3::new(0., 2., 0.),
    );
    let n = cube().vertices.len();
    assert_eq!(m.vertices.len(), plain.vertices.len());
    assert_eq!(&m.vertices[..n], &plain.vertices[..n], "the base stays");
    for (a, b) in m.vertices[n..].iter().zip(&plain.vertices[n..]) {
        assert!(a.distance(frame.transform_point3(*b)) < 1e-9);
    }
}

#[test]
fn tool_pipe_steps_alone_pass_the_mesh_through_and_check_values() {
    for id in ["symmetry.tool.item", "pmodel.workplane.item"] {
        let (mut e, item) = with_base(cube());
        add(&mut e, item, id, json!({}));
        assert_eq!(ok(&e, item), cube());
    }
    let (mut e, item) = with_base(cube());
    add(&mut e, item, "symmetry.tool.item", json!({ "axis": 3 }));
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap();
    assert!(matches!(
        r.statuses[0],
        operandi_core::mesh_stack::OpStatus::Failed(_)
    ));
}
