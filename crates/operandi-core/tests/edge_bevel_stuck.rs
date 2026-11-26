//! The tester's BevelStuck scene (2026-11-25): a cube whose front was inset
//! and a stepped hole with small rounded corners cut through the inset. The
//! loop around the hole's front was bevelled. Unlike the top hole
//! (edge_bevel_hole.rs), two of the old corners sit on the inset's straight
//! lines, so no edge leads away from the bevel, and Loop Slide off couldn't
//! push them; and the rounds' wide corners were refused at a third of
//! their edges' length.
use glam::DVec3;
use operandi_core::*;

/// The scene's cube (fixtures/bevel_stuck_cube.json) and its hole's front
/// loop.
fn cube() -> (Mesh, Vec<(u32, u32)>) {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/bevel_stuck_cube.json")).unwrap();
    let mut m = Mesh::default();
    for p in data["v"].as_array().unwrap() {
        let c: Vec<f64> = p
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        m.vertices.push(DVec3::new(c[0], c[1], c[2]));
    }
    let faces: Vec<Face> = data["f"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| Face {
            vertices: f
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as u32)
                .collect(),
            uv: None,
            material: String::new(),
            smooth: false,
            part: None,
            material_label: None,
        })
        .collect();
    m.faces = faces.into();
    m.validate().unwrap();
    let loop_v = [
        142u32, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 174,
        176, 178, 180, 182, 184, 186, 188,
    ];
    let mut edges = Vec::new();
    for f in m.faces.iter() {
        let v = &f.vertices;
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if loop_v.contains(&a) && loop_v.contains(&b) {
                edges.push((a.min(b), a.max(b)));
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    assert_eq!(edges.len(), 24);
    (m, edges)
}

/// The old corners next to the loop: two at the inset's corners (with an
/// edge out to the cube's corners) and two on its straight lines.
const OLD: [usize; 4] = [137, 138, 140, 141];

fn newell(m: &Mesh, f: &[u32]) -> DVec3 {
    let p = |i: u32| m.vertices[i as usize];
    (0..f.len())
        .map(|k| p(f[k]).cross(p(f[(k + 1) % f.len()])))
        .sum()
}

#[test]
fn the_front_hole_bevels_and_pushes_the_old_corners_away() {
    let (m, edges) = cube();
    for loop_slide in [false, true] {
        let mut last = [0.; 4];
        // Past about 0.025 the step's small rounds (about 0.025 across)
        // close to a point on the strip's inner side instead of folding.
        // (With Loop Slide on, nothing moves out of the way, and past 0.09
        // the strip runs out of edge to slide along.)
        for k in 1..=if loop_slide { 18 } else { 48 } {
            let w = k as f64 * 0.005;
            let what = format!("Loop Slide {loop_slide}, width {w}");
            let settings = EdgeBevelSettings {
                width: w,
                rows: 4,
                depth: 100.,
                chamfer: false,
                options: EdgeBevelOptions {
                    loop_slide,
                    ..Default::default()
                },
            };
            let out = edge_bevel_with(&m, &edges, &settings, None)
                .unwrap_or_else(|e| panic!("{what}: {e}"))
                .mesh;
            out.validate().unwrap();
            // Every polygon lying on the front faces out of it (-z) and
            // doesn't cross itself (one did with Loop Slide on, and drew
            // over the hole).
            let p = |i: u32| out.vertices[i as usize];
            for f in out.faces.iter() {
                let v = &f.vertices;
                let n = v.len();
                if !v.iter().all(|&i| (p(i).z + 0.5).abs() < 1e-9) {
                    continue;
                }
                assert!(
                    newell(&out, v).z < 0.,
                    "{what}: a front polygon flipped: {v:?}"
                );
                // No two corners in one place (a round's points closed onto
                // one spot drew the polygon over the hole).
                for i in 0..n {
                    for j in i + 1..n {
                        assert!(
                            p(v[i]).distance(p(v[j])) > 1e-9,
                            "{what}: a front polygon has two corners at {}",
                            p(v[i])
                        );
                    }
                }
                for i in 0..n {
                    for j in i + 2..n {
                        assert!(
                            (i == 0 && j == n - 1)
                                || !crosses(p(v[i]), p(v[(i + 1) % n]), p(v[j]), p(v[(j + 1) % n])),
                            "{what}: a front polygon crosses itself: {v:?}"
                        );
                    }
                }
            }
            for (j, &i) in OLD.iter().enumerate() {
                let moved = out.vertices[i].distance(m.vertices[i]);
                if loop_slide {
                    assert!(moved < 1e-12, "{what}: point {i} moved");
                } else {
                    // (Past 0.18 point 140 has reached the cube's edge and
                    // slides sideways along it; past 0.24 the strip is wider
                    // than the face.)
                    assert!(
                        w > 0.18 || moved >= last[j] - 1e-9,
                        "{what}: point {i} went back"
                    );
                    last[j] = moved;
                }
            }
        }
        if !loop_slide {
            for (j, &i) in OLD.iter().enumerate() {
                assert!(
                    last[j] > 0.01,
                    "Loop Slide off: point {i} never moved out of the way"
                );
            }
        }
    }
}

/// Whether two segments in the front plane (z = -0.5) cross.
fn crosses(a: DVec3, b: DVec3, c: DVec3, d: DVec3) -> bool {
    let side = |o: DVec3, p: DVec3, q: DVec3| (p.x - o.x) * (q.y - o.y) - (p.y - o.y) * (q.x - o.x);
    let (d1, d2, d3, d4) = (side(c, d, a), side(c, d, b), side(a, b, c), side(a, b, d));
    (d1 > 0.) != (d2 > 0.)
        && (d3 > 0.) != (d4 > 0.)
        && [d1, d2, d3, d4].iter().all(|x| x.abs() > 1e-14)
}

/// Segments replaced Round Level (the tester, 2026-11-25: "0 means flat"),
/// so a bevel can now have any number of rows, odd ones too (Round Level
/// only gave even counts). Every corner pattern, at 1 to 8 rows, on the
/// hole's front loop and on a box corner where three edges meet.
#[test]
fn any_number_of_rows_bevels_with_every_corner_pattern() {
    let (m, edges) = cube();
    let b = Mesh::box_mesh(DVec3::ZERO, DVec3::ONE, 0).unwrap();
    let corner = b
        .vertices
        .iter()
        .position(|p| p.x > 0.4 && p.y > 0.4 && p.z > 0.4)
        .unwrap() as u32;
    let box_edges: Vec<(u32, u32)> = b
        .edges()
        .into_iter()
        .filter(|&(x, y)| x == corner || y == corner)
        .collect();
    assert_eq!(box_edges.len(), 3);
    for pattern in [
        CornerPattern::NoMitering,
        CornerPattern::Balanced,
        CornerPattern::Patch,
        CornerPattern::Uniform,
        CornerPattern::Pinch,
        CornerPattern::Radial,
    ] {
        for rows in 1..=8 {
            for loop_slide in [false, true] {
                for (mesh, sel, w) in [
                    (&m, &edges, 0.02),
                    (&m, &edges, 0.06),
                    (&b, &box_edges, 0.2),
                ] {
                    let what =
                        format!("{pattern:?}, {rows} rows, Loop Slide {loop_slide}, width {w}");
                    let settings = EdgeBevelSettings {
                        width: w,
                        rows,
                        depth: 100.,
                        chamfer: false,
                        options: EdgeBevelOptions {
                            loop_slide,
                            corner: pattern,
                            ..Default::default()
                        },
                    };
                    let out = edge_bevel_with(mesh, sel, &settings, None)
                        .unwrap_or_else(|e| panic!("{what}: {e}"))
                        .mesh;
                    out.validate().unwrap_or_else(|e| panic!("{what}: {e}"));
                    assert!(
                        out.vertices.iter().all(|p| p.is_finite()),
                        "{what}: a point went to infinity"
                    );
                }
            }
        }
    }
}
