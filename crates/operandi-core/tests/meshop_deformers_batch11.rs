//! Deformer steps in the Mesh Stack (batch 11): Weight, Push and Morph
//! Influence read the mesh's vertex maps.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use glam::DVec3;
use operandi_core::mesh_stack::OpStatus;
use operandi_core::*;
use serde_json::{Value, json};

/// The cube with weight map "Weight" (point 0 at 1, point 1 at 0.5) and
/// morph map "Morph" (point 2 up by 1).
fn mapped() -> Mesh {
    let mut m = cube();
    m.attributes
        .point_weights
        .insert("Weight".into(), [(0, 1.), (1, 0.5)].into());
    m.attributes
        .relative_morphs
        .insert("Morph".into(), [(2, DVec3::Y)].into());
    m
}
fn run(id: &str, values: Value) -> Mesh {
    let (mut e, item) = with_base(mapped());
    add(&mut e, item, id, values);
    ok(&e, item)
}
fn fails(id: &str, values: Value) {
    let (mut e, item) = with_base(mapped());
    add(&mut e, item, id, values);
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap();
    assert!(matches!(r.statuses[0], OpStatus::Failed(_)), "{id}");
}

#[test]
fn weight_influence_moves_by_the_maps_weight() {
    let base = mapped();
    let m = run("deform.weightInfluence.item", json!({}));
    for (i, (p, was)) in m.vertices.iter().zip(&base.vertices).enumerate() {
        let w = [1., 0.5].get(i).copied().unwrap_or(0.);
        assert!(p.distance(*was + DVec3::Y * 0.5 * w) < 1e-9, "{i}");
    }
}

#[test]
fn push_influence_pushes_along_point_normals() {
    let base = mapped();
    let m = run("deform.pushInfluence.item", json!({ "distance": 0.2 }));
    for (p, was) in m.vertices.iter().zip(&base.vertices) {
        // A cube corner's normal points out from the centre.
        assert!(p.distance(*was + was.normalize() * 0.2) < 1e-9);
    }
    // With the map: only the weighted points move.
    let m = run(
        "deform.pushInfluence.item",
        json!({ "distance": 0.2, "mapName": "Weight" }),
    );
    assert!(m.vertices[2..] == base.vertices[2..]);
    assert!(m.vertices[1].distance(base.vertices[1]) > 0.09);
}

#[test]
fn morph_influence_dials_in_the_morph() {
    let base = mapped();
    let m = run("deform.morphInfluence.item", json!({ "strength": 0.5 }));
    for (i, (p, was)) in m.vertices.iter().zip(&base.vertices).enumerate() {
        let want = if i == 2 { *was + DVec3::Y * 0.5 } else { *was };
        assert!(p.distance(want) < 1e-9);
    }
}

#[test]
fn influences_need_their_maps() {
    fails("deform.weightInfluence.item", json!({ "mapName": "Nope" }));
    fails("deform.pushInfluence.item", json!({ "mapName": "Nope" }));
    fails("deform.morphInfluence.item", json!({ "mapName": "Nope" }));
}
