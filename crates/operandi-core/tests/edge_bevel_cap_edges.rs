//! Edge Bevel at an end point where more polygons meet than the strip's
//! two sides and one end polygon (tester's report, 2026-11-24: the tip and
//! inner corners of a pointed shape whose top was split into triangles
//! were left as spikes). The end's rounded profile crosses the extra edges;
//! each must end on it, and the old point must go.
use glam::DVec3;
use operandi_core::*;

fn mesh(points: &[[f64; 3]], faces: &[&[u32]]) -> Mesh {
    Mesh {
        vertices: points.iter().map(|&p| DVec3::from_array(p)).collect(),
        faces: faces
            .iter()
            .map(|f| Face {
                vertices: f.to_vec(),
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

/// A 1 m box (as edge_bevel_options' cube) whose top is a fan of
/// triangles from corner 7 to the points listed in `fan`.
const BOX: [[f64; 3]; 8] = [
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
];

/// Top split by the diagonal 7–5: one extra edge into corner 7.
fn diagonal_top() -> Mesh {
    mesh(
        &BOX,
        &[
            &[0, 1, 2, 3],
            &[7, 6, 5],
            &[7, 5, 4],
            &[0, 4, 5, 1],
            &[1, 5, 6, 2],
            &[2, 6, 7, 3],
            &[3, 7, 4, 0],
        ],
    )
}

/// Top split into three triangles from 7, through 5 and a point 8 halfway
/// along 5–6: two extra edges into corner 7.
fn fan_top() -> Mesh {
    let mut points = BOX.to_vec();
    points.push([0.5, 0.5, 0.]);
    mesh(
        &points,
        &[
            &[0, 1, 2, 3],
            &[7, 6, 8],
            &[7, 8, 5],
            &[7, 5, 4],
            &[0, 4, 5, 1],
            &[1, 5, 8, 6, 2],
            &[2, 6, 7, 3],
            &[3, 7, 4, 0],
        ],
    )
}

fn bevel(m: &Mesh, edges: &[(u32, u32)], width: f64, rows: u32, support: f64) -> EdgeBevelResult {
    let settings = EdgeBevelSettings {
        width,
        rows,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            support,
            ..Default::default()
        },
    };
    edge_bevel_with(m, edges, &settings, None).unwrap()
}

/// No polygon still uses the selected edges' end points, the mesh is
/// valid, and the top stays flat (every point of every top polygon at
/// y = 0.5): no spike reaches back to the old corner.
fn check(name: &str, before: &Mesh, edges: &[(u32, u32)], out: &EdgeBevelResult) {
    let m = &out.mesh;
    m.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
    // Points are renumbered afterwards, so look for the old corners by place.
    for &(a, b) in edges {
        for v in [a, b] {
            let at = before.vertices[v as usize];
            let users: Vec<usize> = (0..m.faces.len())
                .filter(|&f| {
                    m.faces[f]
                        .vertices
                        .iter()
                        .any(|&w| m.vertices[w as usize].distance(at) < 1e-9)
                })
                .collect();
            assert!(
                users.is_empty(),
                "{name}: the old corner {at} is still used by {users:?}"
            );
        }
    }
    let top = |f: &Face| {
        f.vertices
            .iter()
            .all(|&v| (m.vertices[v as usize].y - 0.5).abs() < 1e-9)
    };
    let tops_before = before.faces.iter().filter(|f| {
        f.vertices
            .iter()
            .all(|&v| (before.vertices[v as usize].y - 0.5).abs() < 1e-9)
    });
    assert_eq!(
        m.faces.iter().filter(|f| top(f)).count(),
        tops_before.count(),
        "{name}: each top polygon keeps its place, flat"
    );
    // Every new point of a top polygon lies within the original top square.
    for f in m.faces.iter().filter(|f| top(f)) {
        for &v in &f.vertices {
            let p = m.vertices[v as usize];
            assert!(
                p.x.abs() <= 0.5 + 1e-9 && p.z.abs() <= 0.5 + 1e-9,
                "{name}: {p} is outside the top"
            );
        }
    }
}

#[test]
fn extra_edges_into_the_corner_end_on_the_profile() {
    for (name, m) in [("diagonal", diagonal_top()), ("fan", fan_top())] {
        for rows in [1, 2, 4, 7] {
            let what = format!("{name}, {rows} rows");
            let out = bevel(&m, &[(3, 7)], 0.2, rows, 0.);
            check(&what, &m, &[(3, 7)], &out);
            // Each extra edge from 7 now ends on the profile: a point on
            // the line from 7 toward 5 (and 8), 0.2 m or less from 7.
            let ends: Vec<u32> = if name == "diagonal" {
                vec![5]
            } else {
                vec![5, 8]
            };
            for s in ends {
                let (a, b) = (DVec3::new(-0.5, 0.5, 0.5), m.vertices[s as usize]);
                let dir = (b - a).normalize();
                let on = out.mesh.vertices.iter().any(|&p| {
                    let t = (p - a).dot(dir);
                    t > 1e-6 && t < 0.2 + 1e-6 && (a + dir * t).distance(p) < 1e-9
                });
                assert!(
                    on,
                    "{what}: no point where the edge 7–{s} meets the profile"
                );
            }
        }
    }
}

#[test]
fn with_mitering_too() {
    for (name, m) in [("diagonal", diagonal_top()), ("fan", fan_top())] {
        let out = bevel(&m, &[(3, 7)], 0.2, 4, 0.1);
        out.mesh
            .validate()
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let all = out.mesh.edges();
        for e in out.selected_after.iter().chain(&out.strip_edges) {
            assert!(all.binary_search(e).is_ok(), "{name}: {e:?} isn't an edge");
        }
        let corner = DVec3::new(-0.5, 0.5, 0.5);
        assert!(
            !out.mesh.faces.iter().any(|f| f
                .vertices
                .iter()
                .any(|&w| out.mesh.vertices[w as usize].distance(corner) < 1e-9)),
            "{name}: the old corner 7 is still used"
        );
    }
}

#[test]
fn a_plain_box_corner_is_unchanged() {
    // Without extra edges the result is what it always was.
    let plain = mesh(
        &BOX,
        &[
            &[0, 1, 2, 3],
            &[7, 6, 5, 4],
            &[0, 4, 5, 1],
            &[1, 5, 6, 2],
            &[2, 6, 7, 3],
            &[3, 7, 4, 0],
        ],
    );
    let out = bevel(&plain, &[(3, 7)], 0.2, 4, 0.);
    check("plain", &plain, &[(3, 7)], &out);
    // Two ends of 2 side points and 3 profile rows each replace 2 corners.
    assert_eq!(out.mesh.vertices.len(), 8 - 2 + 2 * 5);
    assert_eq!(out.mesh.faces.len(), 6 + 4);
}
