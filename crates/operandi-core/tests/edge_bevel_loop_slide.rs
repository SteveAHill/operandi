//! Edge Bevel's Loop Slide (a tester's request, 2026-11-24, from Blender
//! screenshots): a box with an inset top pushed down into a hole, the
//! hole's rim bevelled. The inset's diagonals run between two bevelled rim
//! edges into each corner. On (the default), the new point slides along
//! the diagonal, which stays straight. Off, it goes where the two strips'
//! sides meet, the width from both rim edges, and the diagonal bends to it.
use glam::DVec3;
use operandi_core::*;

/// Outer top corners 0–3, hole rim 4–7 (an uneven inset, so the diagonals
/// aren't the corners' bisectors), hole floor 8–11, box bottom 12–15.
pub(crate) fn hole() -> Mesh {
    let o = [[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]];
    let i = [[-0.4, -0.2], [0.6, -0.2], [0.6, 0.5], [-0.4, 0.5]];
    let mut points = Vec::new();
    for (ring, y) in [(&o, 1.), (&i, 1.), (&i, 0.4), (&o, -1.)] {
        for [x, z] in ring.iter() {
            points.push(DVec3::new(*x, y, *z));
        }
    }
    // Around the top counter-clockwise seen from above.
    let s = [0u32, 3, 2, 1];
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for k in 0..4 {
        let (a, b) = (s[k], s[(k + 1) % 4]);
        faces.push(vec![a, b, 4 + b, 4 + a]); // top ring
        faces.push(vec![4 + a, 4 + b, 8 + b, 8 + a]); // hole wall
        faces.push(vec![b, a, 12 + a, 12 + b]); // box side
    }
    faces.push(s.iter().map(|k| 8 + k).collect()); // hole floor
    faces.push(s.iter().rev().map(|k| 12 + k).collect()); // box bottom
    Mesh {
        vertices: points,
        faces: faces
            .into_iter()
            .map(|vertices| Face {
                vertices,
                uv: None,
                material: String::new(),
                smooth: false,
                part: None,
                material_label: None,
            })
            .collect(),
        ..Default::default()
    }
}

const RIM: [(u32, u32); 4] = [(4, 5), (5, 6), (6, 7), (7, 4)];
const WIDTH: f64 = 0.1;

fn bevel(loop_slide: bool, rows: u32) -> Mesh {
    let m = hole();
    m.validate().unwrap();
    let settings = EdgeBevelSettings {
        width: WIDTH,
        rows,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            loop_slide,
            ..Default::default()
        },
    };
    let out = edge_bevel_with(&m, &RIM, &settings, None).unwrap();
    out.mesh.validate().unwrap();
    out.mesh
}

/// The point each outer top corner's diagonal now leads to (the other end
/// of its edge on the top, y = 1, that isn't on the outer square).
fn diagonal_end(m: &Mesh, outer: DVec3) -> DVec3 {
    let at = |p: DVec3| {
        m.vertices
            .iter()
            .position(|&q| q.distance(p) < 1e-9)
            .unwrap() as u32
    };
    let o = at(outer);
    let ends: Vec<DVec3> = m
        .edges()
        .into_iter()
        .filter_map(|(a, b)| {
            let other = if a == o {
                b
            } else if b == o {
                a
            } else {
                return None;
            };
            let q = m.vertices[other as usize];
            ((q.y - 1.).abs() < 1e-9 && q.x.abs() < 1. - 1e-9 && q.z.abs() < 1. - 1e-9).then_some(q)
        })
        .collect();
    assert_eq!(ends.len(), 1, "one diagonal from {outer}: {ends:?}");
    ends[0]
}

/// Distance from `q` to the line through `a` and `b`.
fn off_line(q: DVec3, a: DVec3, b: DVec3) -> f64 {
    let d = (b - a).normalize();
    let v = q - a;
    (v - d * v.dot(d)).length()
}

#[test]
fn loop_slide_on_keeps_the_diagonals_straight() {
    let before = hole();
    for rows in [1, 2, 4] {
        let m = bevel(true, rows);
        for k in 0..4 {
            let (outer, rim) = (before.vertices[k], before.vertices[4 + k]);
            let end = diagonal_end(&m, outer);
            assert!(
                off_line(end, outer, rim) < 1e-9,
                "rows {rows}: corner {k}'s diagonal bends to {end}"
            );
            assert!(
                end.distance(outer) < rim.distance(outer),
                "it ends short of the rim corner"
            );
        }
    }
}

#[test]
fn loop_slide_off_gives_an_even_width_and_the_diagonals_bend() {
    let before = hole();
    let p = |i: usize| before.vertices[i];
    for rows in [1, 2, 4] {
        let m = bevel(false, rows);
        for k in 0..4 {
            let end = diagonal_end(&m, p(k));
            // The rim edges at corner k: to the previous and next rim corner.
            let (prev, next) = (4 + (k + 3) % 4, 4 + (k + 1) % 4);
            for other in [prev, next] {
                let d = off_line(end, p(4 + k), p(other));
                assert!(
                    (d - WIDTH).abs() < 1e-9,
                    "rows {rows}: corner {k} is {d} from rim edge {}-{other}, want {WIDTH}",
                    4 + k
                );
            }
            assert!(
                off_line(end, p(k), p(4 + k)) > 1e-3,
                "rows {rows}: corner {k}'s diagonal should bend (uneven inset)"
            );
        }
    }
}

#[test]
fn loop_slide_is_on_by_default() {
    // The default options give exactly the Loop Slide on result, the
    // measured one (EBC-B3-03; the maintainer, 2026-11-26).
    let m = hole();
    let settings = EdgeBevelSettings {
        width: WIDTH,
        rows: 2,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions::default(),
    };
    assert!(
        EdgeBevelOptions::default().loop_slide,
        "Loop Slide is on by default"
    );
    let a = edge_bevel_with(&m, &RIM, &settings, None).unwrap().mesh;
    let b = bevel(true, 2);
    assert_eq!(a.vertices, b.vertices);
}
