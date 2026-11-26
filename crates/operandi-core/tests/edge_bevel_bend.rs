//! The tester's bent chain (2026-11-24 picture): a box with its top front
//! edge cut at an angle and a loop cut around it. The right front vertical
//! edge, the cut's right edge and the top right edges are bevelled, a chain
//! that bends 45° twice. The strip kept the width along each edge only at
//! square corners: at these bends its corner point sat about 70% of the
//! width from the edges, so the strip narrowed and kinked.
use glam::DVec3;
use operandi_core::*;

const X: f64 = 0.6;
/// Around the solid in (z, y): front, over the top, back, under.
const PROFILE: [(f64, f64); 7] = [
    (1.0, 0.0),
    (1.0, 0.8),
    (0.6, 1.2),
    (0.2, 1.2),
    (-1.5, 1.2),
    (-1.5, 0.0),
    (0.2, 0.0),
];

fn face(m: &Mesh, mut v: Vec<u32>) -> Face {
    let p = |i: u32| m.vertices[i as usize];
    let mut n = DVec3::ZERO;
    for k in 0..v.len() {
        n += p(v[k]).cross(p(v[(k + 1) % v.len()]));
    }
    let centre = v.iter().map(|&i| p(i)).sum::<DVec3>() / v.len() as f64;
    let solid = DVec3::new(0., 0.6, -0.25);
    if n.dot(centre - solid) < 0. {
        v.reverse();
    }
    Face {
        vertices: v,
        uv: None,
        material: String::new(),
        smooth: false,
        part: None,
        material_label: None,
    }
}

fn shape() -> Mesh {
    let mut m = Mesh::default();
    for x in [X, -X] {
        for (z, y) in PROFILE {
            m.vertices.push(DVec3::new(x, y, z));
        }
    }
    let (r, l) = (|k: u32| k, |k: u32| k + 7);
    let mut faces = Vec::new();
    for side in [r, l] {
        faces.push(face(&m, vec![side(0), side(1), side(2), side(3), side(6)]));
        faces.push(face(&m, vec![side(6), side(3), side(4), side(5)]));
    }
    let ring = [0, 1, 2, 3, 4, 5, 6, 0];
    for w in ring.windows(2) {
        faces.push(face(&m, vec![r(w[0]), r(w[1]), l(w[1]), l(w[0])]));
    }
    m.faces = faces.into();
    m
}

/// Distance from `q` to the line through `a` and `b`.
fn off(q: DVec3, a: DVec3, b: DVec3) -> f64 {
    (q - a).cross((b - a).normalize()).length()
}

#[test]
fn the_strip_keeps_its_width_through_the_bends() {
    let m = shape();
    m.validate().unwrap();
    let key = |a: u32, b: u32| (a.min(b), a.max(b));
    let edges = vec![key(0, 1), key(1, 2), key(2, 3), key(3, 4)];
    let w = 0.25;
    // Every Corner Pattern: the bend's point comes before any corner fill.
    for (rows, corner) in [1u32, 4].into_iter().flat_map(|r| {
        [
            CornerPattern::NoMitering,
            CornerPattern::Balanced,
            CornerPattern::Patch,
            CornerPattern::Uniform,
            CornerPattern::Pinch,
            CornerPattern::Radial,
        ]
        .map(|c| (r, c))
    }) {
        let settings = EdgeBevelSettings {
            width: w,
            rows,
            depth: 100.,
            chamfer: false,
            // Loop Slide off keeps the width through the bends; on (the
            // default) slides it along the edges as measured (EBC-B3-03).
            options: EdgeBevelOptions {
                corner,
                loop_slide: false,
                ..Default::default()
            },
        };
        let out = edge_bevel_with(&m, &edges, &settings, None).unwrap().mesh;
        out.validate().unwrap();
        let at = |i: usize| DVec3::new(X, PROFILE[i].1, PROFILE[i].0);
        // Each bend's point on the right face: a full width from both edges.
        for (before, bend, after) in [(0, 1, 2), (1, 2, 3)] {
            let (a, v, b) = (at(before), at(bend), at(after));
            let mitre = out
                .vertices
                .iter()
                .filter(|q| {
                    (q.x - X).abs() < 1e-9 && q.distance(v) < 2. * w && q.distance(v) > 1e-6
                })
                .find(|&&q| (off(q, a, v) - w).abs() < 1e-9 && (off(q, v, b) - w).abs() < 1e-9);
            assert!(
                mitre.is_some(),
                "rows {rows}, {corner:?}: the bend at {v} narrows the strip"
            );
        }
    }
}
