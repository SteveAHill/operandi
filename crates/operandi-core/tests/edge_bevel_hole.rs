//! The tester's square hole (2026-11-24 pictures): a cube whose top was
//! inset and the inner square extruded down into a hole. The hole's four
//! vertical corner edges were bevelled round, and then the loop around the
//! hole's top (straight runs and the rounded corners) was bevelled.
use glam::DVec3;
use operandi_core::*;

fn face(v: Vec<u32>) -> Face {
    Face {
        vertices: v,
        uv: None,
        material: String::new(),
        smooth: false,
        part: None,
        material_label: None,
    }
}

/// Points: cube top corners 0..4, its bottom 4..8, the rim (inset square at
/// the top) 8..12, the hole's floor 12..16; each square runs around the
/// same way. Polygons face out of the solid.
pub(crate) fn holed_cube() -> Mesh {
    let mut m = Mesh::default();
    let square =
        |h: f64, y: f64| [(-h, -h), (h, -h), (h, h), (-h, h)].map(|(x, z)| DVec3::new(x, y, z));
    for (h, y) in [(1., 1.), (1., -1.), (0.6, 1.), (0.6, 0.)] {
        m.vertices.extend(square(h, y));
    }
    let mut faces = Vec::new();
    let next = |i: u32| (i + 1) % 4;
    for i in 0..4 {
        // Cube sides, the top's inset ring, the hole's walls.
        faces.push(vec![i, 4 + i, 4 + next(i), next(i)]);
        faces.push(vec![i, next(i), 8 + next(i), 8 + i]);
        faces.push(vec![8 + i, 8 + next(i), 12 + next(i), 12 + i]);
    }
    faces.push(vec![4, 7, 6, 5]);
    faces.push(vec![12, 13, 14, 15]);
    // Wind them all one way (each shared edge runs opposite ways in its
    // two polygons), then outward (positive volume).
    let mut done = vec![false; faces.len()];
    done[0] = true;
    let mut queue = vec![0usize];
    while let Some(f) = queue.pop() {
        let fe: Vec<(u32, u32)> = (0..faces[f].len())
            .map(|k| (faces[f][k], faces[f][(k + 1) % faces[f].len()]))
            .collect();
        for g in 0..faces.len() {
            if done[g] {
                continue;
            }
            let ge: Vec<(u32, u32)> = (0..faces[g].len())
                .map(|k| (faces[g][k], faces[g][(k + 1) % faces[g].len()]))
                .collect();
            let same = ge.iter().any(|e| fe.contains(e));
            let opposite = ge.iter().any(|&(a, b)| fe.contains(&(b, a)));
            if same || opposite {
                if same {
                    faces[g].reverse();
                }
                done[g] = true;
                queue.push(g);
            }
        }
    }
    let p = |i: u32| m.vertices[i as usize];
    let volume: f64 = faces
        .iter()
        .map(|v| {
            (1..v.len() - 1)
                .map(|k| p(v[0]).dot(p(v[k]).cross(p(v[k + 1]))))
                .sum::<f64>()
        })
        .sum();
    for mut v in faces {
        if volume < 0. {
            v.reverse();
        }
        m.faces.push(face(v));
    }
    m
}

fn key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

fn bevel(m: &Mesh, edges: &[(u32, u32)], width: f64, rows: u32) -> Mesh {
    let settings = EdgeBevelSettings {
        width,
        rows,
        depth: 100.,
        chamfer: false,
        options: Default::default(),
    };
    let out = edge_bevel_with(m, edges, &settings, None).unwrap().mesh;
    out.validate().unwrap();
    out
}

/// The rounded hole: its four vertical corners bevelled round.
pub(crate) fn rounded_hole() -> Mesh {
    let m = holed_cube();
    m.validate().unwrap();
    let corners: Vec<(u32, u32)> = (0..4).map(|i| key(8 + i, 12 + i)).collect();
    bevel(&m, &corners, 0.2, 4)
}

/// The loop around the hole's top: edges on the top whose two polygons are
/// the top and a wall going down.
pub(crate) fn rim(m: &Mesh) -> Vec<(u32, u32)> {
    let p = |i: u32| m.vertices[i as usize];
    let mut e: Vec<(u32, u32)> = m
        .faces
        .iter()
        .filter(|f| f.vertices.iter().any(|&i| p(i).y < 1. - 1e-9))
        .flat_map(|f| {
            let v = &f.vertices;
            (0..v.len()).map(move |k| key(v[k], v[(k + 1) % v.len()]))
        })
        .filter(|&(a, b)| {
            let inside = |q: DVec3| q.x.abs() < 0.6 + 1e-9 && q.z.abs() < 0.6 + 1e-9;
            (p(a).y - 1.).abs() < 1e-9 && (p(b).y - 1.).abs() < 1e-9 && inside(p(a)) && inside(p(b))
        })
        .collect();
    e.sort_unstable();
    e.dedup();
    e
}

/// Every polygon of `out` faces the way the polygon it came from on the
/// surface faces: the top's up, a wall's sideways.
fn faces_agree(out: &Mesh) -> Vec<String> {
    let p = |i: u32| out.vertices[i as usize];
    let mut bad = Vec::new();
    for (fi, f) in out.faces.iter().enumerate() {
        let mut n = DVec3::ZERO;
        for k in 0..f.vertices.len() {
            n += p(f.vertices[k]).cross(p(f.vertices[(k + 1) % f.vertices.len()]));
        }
        let c = f.vertices.iter().map(|&i| p(i)).sum::<DVec3>() / f.vertices.len() as f64;
        // Polygons lying on the top (all points at y = 1) must face up.
        if f.vertices.iter().all(|&i| (p(i).y - 1.).abs() < 1e-9) && n.y <= 1e-12 {
            bad.push(format!("top polygon {fi} at {c} faces {n}"));
        }
    }
    bad
}

#[test]
fn the_rounded_hole_is_valid_and_flat_on_top() {
    let m = rounded_hole();
    if let Ok(path) = std::env::var("OPERANDI_DUMP_HOLE") {
        let v: Vec<String> = m
            .vertices
            .iter()
            .map(|q| format!("[{},{},{}]", q.x, q.y, q.z))
            .collect();
        let f: Vec<String> = m
            .faces
            .iter()
            .map(|f| format!("{:?}", f.vertices))
            .collect();
        std::fs::write(
            format!("{path}-round.json"),
            format!("{{\"v\":[{}],\"f\":[{}]}}", v.join(","), f.join(",")),
        )
        .unwrap();
    }
    assert!(faces_agree(&m).is_empty(), "{:?}", faces_agree(&m));
    let r = rim(&m);
    assert_eq!(r.len(), 4 + 4 * 4, "rim: four runs and four rounds of four");
}

/// The rim bevelled `w` wide with `rows` rows, Loop Slide on or off.
fn rim_bevel(m: &Mesh, w: f64, rows: u32, loop_slide: bool) -> Result<Mesh> {
    let settings = EdgeBevelSettings {
        width: w,
        rows,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            loop_slide,
            ..Default::default()
        },
    };
    let out = edge_bevel_with(m, &rim(m), &settings, None)?.mesh;
    out.validate()?;
    Ok(out)
}

fn dump(out: &Mesh, name: &str) {
    if let Ok(path) = std::env::var("OPERANDI_DUMP_HOLE") {
        let v: Vec<String> = out
            .vertices
            .iter()
            .map(|q| format!("[{},{},{}]", q.x, q.y, q.z))
            .collect();
        let f: Vec<String> = out
            .faces
            .iter()
            .map(|f| format!("{:?}", f.vertices))
            .collect();
        std::fs::write(
            format!("{path}-{name}.json"),
            format!("{{\"v\":[{}],\"f\":[{}]}}", v.join(","), f.join(",")),
        )
        .unwrap();
    }
}

/// The four old square corners (points 8..12) and their rounds' centres.
const CORNERS: [(usize, f64, f64); 4] = [(8, -1., -1.), (9, 1., -1.), (10, 1., 1.), (11, -1., 1.)];

/// Every top point near a rounded corner lies inside the round or at least
/// `w` outside it: the rim's strip keeps its full width there.
fn full_width(out: &Mesh, w: f64) -> Option<DVec3> {
    CORNERS.iter().find_map(|&(_, sx, sz)| {
        let centre = DVec3::new(0.4 * sx, 1., 0.4 * sz);
        out.vertices.iter().copied().find(|p| {
            let r = (*p - centre).length();
            (p.y - 1.).abs() < 1e-9
                && (p.x - centre.x) * sx > 0.
                && (p.z - centre.z) * sz > 0.
                && r > 0.2 + 1e-6
                && r < 0.2 + w - 1e-6
        })
    })
}

#[test]
fn bevelling_the_rounded_rim_keeps_the_top_clean() {
    let m = rounded_hole();
    for loop_slide in [false, true] {
        for rows in [1u32, 4] {
            let what = format!("Loop Slide {loop_slide}, rows {rows}");
            let out = rim_bevel(&m, 0.05, rows, loop_slide).unwrap();
            dump(
                &out,
                &format!("{}{rows}", if loop_slide { "on" } else { "off" }),
            );
            let bad = faces_agree(&out);
            assert!(bad.is_empty(), "{what}: {bad:?}");
        }
    }
}

/// The tester (2026-11-24): with Loop Slide off the bevel comes first, at
/// its full width from the start, and the geometry in its way adapts: the
/// old square corners slide out along their diagonals, smoothly and
/// steadily as the width grows (they once snapped back: "it bugs out and
/// resets and works differently"), and wide bevels still work.
#[test]
fn loop_slide_off_moves_the_corners_out_of_the_way() {
    let m = rounded_hole();
    let mut last = 0.6;
    for k in 1..=40 {
        let w = k as f64 * 0.01;
        let out = rim_bevel(&m, w, 4, false).unwrap_or_else(|e| panic!("width {w}: {e}"));
        if [5, 15, 30].contains(&k) {
            dump(&out, &format!("off-w{k}"));
        }
        let bad = faces_agree(&out);
        assert!(bad.is_empty(), "width {w}: {bad:?}");
        // (Past about 0.36 the corners run out of diagonal and the strip
        // has to narrow by the cube's outer corners.)
        if let Some(p) = full_width(&out, w).filter(|_| w <= 0.3 + 1e-9) {
            panic!("width {w}: the strip narrows at {p}");
        }
        for &(i, sx, sz) in &CORNERS {
            let q = out.vertices[i];
            assert!(
                (q.x * sx - q.z * sz).abs() < 1e-9,
                "width {w}: corner {i} left its diagonal"
            );
        }
        let corner = -out.vertices[8].x;
        assert!(
            corner >= last - 1e-9,
            "width {w}: the corner went back to {corner}"
        );
        assert!(
            corner - last < 0.03,
            "width {w}: the corner jumped to {corner}"
        );
        last = corner;
    }
    assert!(last > 0.6 + 0.1, "the corners never moved");
}

/// With Loop Slide on the bevel adapts to the geometry instead: nothing
/// around it moves, and nothing flips.
#[test]
fn loop_slide_on_leaves_the_corners_alone() {
    let m = rounded_hole();
    for k in 1..=15 {
        let w = k as f64 * 0.01;
        let out = rim_bevel(&m, w, 4, true).unwrap_or_else(|e| panic!("width {w}: {e}"));
        if [5, 15].contains(&k) {
            dump(&out, &format!("on-w{k}"));
        }
        let bad = faces_agree(&out);
        assert!(bad.is_empty(), "width {w}: {bad:?}");
        for &(i, sx, sz) in &CORNERS {
            assert!(
                out.vertices[i].distance(DVec3::new(0.6 * sx, 1., 0.6 * sz)) < 1e-12,
                "width {w}: corner {i} moved"
            );
        }
        // The straight runs keep their width on top: where each meets a
        // round, the rim's edge carries straight on to the old corner, so
        // sliding along it would give the strip no width there.
        let run_end = DVec3::new(-0.4, 1., -0.6 - w);
        assert!(
            out.vertices.iter().any(|p| p.distance(run_end) < 0.2 * w),
            "width {w}: no full-width side point where the run meets the round"
        );
    }
}
