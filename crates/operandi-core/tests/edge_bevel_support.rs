//! Edge Bevel's Support Edges (a tester's request, 2026-11-23): one more row
//! of edges beyond the bevel on each side, in the neighbouring polygons'
//! planes. Its second handle drives it.
use glam::DVec3;
use operandi_core::*;

/// A 2 m box and its top-front edge (both ends at y = 1, z = 1).
fn cube_and_edge() -> (Mesh, (u32, u32)) {
    let mesh = primitive::cube_geometry(DVec3::ZERO, DVec3::splat(2.)).unwrap();
    let v = &mesh.vertices;
    let edge = mesh
        .edges()
        .into_iter()
        .find(|&(a, b)| {
            let (p, q) = (v[a as usize], v[b as usize]);
            p.y > 0. && q.y > 0. && p.z > 0. && q.z > 0.
        })
        .unwrap();
    (mesh, edge)
}

fn bevel(mesh: &Mesh, edge: (u32, u32), width: f64, support: f64) -> Mesh {
    let settings = EdgeBevelSettings {
        width,
        rows: 1,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            support,
            ..Default::default()
        },
    };
    edge_bevel_with(mesh, &[edge], &settings, None)
        .unwrap()
        .mesh
}

/// How many polygons aren't flat.
fn twisted(mesh: &Mesh) -> usize {
    (0..mesh.faces.len())
        .filter(|&f| {
            let p: Vec<DVec3> = mesh.faces[f]
                .vertices
                .iter()
                .map(|&i| mesh.vertices[i as usize])
                .collect();
            let n = mesh.normal(f);
            p.iter().any(|q| (*q - p[0]).dot(n).abs() >= 1e-9)
        })
        .count()
}

fn assert_flat(mesh: &Mesh) {
    for (f, face) in mesh.faces.iter().enumerate() {
        let p: Vec<DVec3> = face
            .vertices
            .iter()
            .map(|&i| mesh.vertices[i as usize])
            .collect();
        let n = mesh.normal(f);
        for q in &p {
            assert!(
                (*q - p[0]).dot(n).abs() < 1e-9,
                "polygon {f} is twisted: {p:?}"
            );
        }
    }
}

#[test]
fn support_edges_add_a_row_beyond_the_bevel_on_each_side() {
    let (mesh, edge) = cube_and_edge();
    assert_flat(&bevel(&mesh, edge, 0.2, 0.));
    let out = bevel(&mesh, edge, 0.2, 0.1);
    assert_flat(&out);
    // At each end of the edge, the points near the old corner: the bevel's
    // at 0.2 and the support row's at 0.3, one on each side edge.
    for end in [edge.0, edge.1] {
        let corner = mesh.vertices[end as usize];
        let mut near: Vec<f64> = out
            .vertices
            .iter()
            .map(|p| p.distance(corner))
            .filter(|&d| d < 0.5)
            .collect();
        near.sort_by(f64::total_cmp);
        let want = [0.2, 0.2, 0.3, 0.3];
        assert_eq!(near.len(), 4, "{near:?}");
        for (got, want) in near.iter().zip(want) {
            assert!((got - want).abs() < 1e-9, "{near:?}");
        }
    }
    // Four more points and four more polygons than the plain bevel.
    let plain = bevel(&mesh, edge, 0.2, 0.);
    assert_eq!(out.vertices.len(), plain.vertices.len() + 4);
    assert_eq!(out.faces.len(), plain.faces.len() + 2);
}

#[test]
fn support_zero_is_the_plain_bevel_and_bad_values_are_refused() {
    let (mesh, edge) = cube_and_edge();
    let settings = |support| EdgeBevelSettings {
        width: 0.2,
        rows: 1,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            support,
            ..Default::default()
        },
    };
    let none = edge_bevel_with(&mesh, &[edge], &settings(0.), None).unwrap();
    let base = EdgeBevelSettings {
        options: Default::default(),
        ..settings(0.)
    };
    assert_eq!(none, edge_bevel_with(&mesh, &[edge], &base, None).unwrap());
    for bad in [-0.1, f64::NAN] {
        assert!(edge_bevel_with(&mesh, &[edge], &settings(bad), None).is_err());
    }
}

/// Two bevelled edges meeting at a corner, flat and rounded: the rows join
/// up and twist nothing.
#[test]
fn support_edges_hold_at_a_corner_and_on_a_rounded_bevel() {
    let (mesh, edge) = cube_and_edge();
    let v = &mesh.vertices;
    // A second top edge from the first edge's end.
    let other = mesh
        .edges()
        .into_iter()
        .find(|&(a, b)| {
            (a, b) != edge
                && (a == edge.1 || b == edge.1 || a == edge.0 || b == edge.0)
                && v[a as usize].y > 0.
                && v[b as usize].y > 0.
        })
        .unwrap();
    for rows in [1, 4] {
        let settings = EdgeBevelSettings {
            width: 0.2,
            rows,
            depth: 100.,
            chamfer: false,
            options: EdgeBevelOptions {
                support: 0.1,
                ..Default::default()
            },
        };
        let out = edge_bevel_with(&mesh, &[edge, other], &settings, None).unwrap();
        out.mesh.validate().unwrap();
        let plain = edge_bevel_with(
            &mesh,
            &[edge, other],
            &EdgeBevelSettings {
                options: Default::default(),
                ..settings.clone()
            },
            None,
        )
        .unwrap();
        assert!(
            out.mesh.faces.len() > plain.mesh.faces.len(),
            "rows {rows}: the support rows split polygons"
        );
        // A flat bevel stays flat; a rounded one's corner patch curves
        // (as without Support Edges), and the rows twist nothing more.
        if rows == 1 {
            assert_flat(&out.mesh);
        } else {
            assert!(twisted(&out.mesh) <= twisted(&plain.mesh), "rows {rows}");
        }
    }
}

/// The inset alone (no bevel): an edge on each side of the selected edge,
/// the edge itself kept.
#[test]
fn inset_alone_adds_an_edge_on_each_side() {
    let (mesh, edge) = cube_and_edge();
    let out = bevel(&mesh, edge, 0., 0.3);
    out.validate().unwrap();
    assert_flat(&out);
    let (a, b) = (
        mesh.vertices[edge.0 as usize],
        mesh.vertices[edge.1 as usize],
    );
    // The selected edge is still there.
    let has = |p: DVec3| out.vertices.iter().any(|q| q.distance(p) < 1e-9);
    assert!(has(a) && has(b), "the selected edge stays");
    // At each end, one new point 0.3 along each side edge.
    for corner in [a, b] {
        let near: Vec<f64> = out
            .vertices
            .iter()
            .map(|p| p.distance(corner))
            .filter(|&d| d > 1e-9 && d < 0.5)
            .collect();
        assert_eq!(near.len(), 2, "{near:?}");
        assert!(near.iter().all(|d| (d - 0.3).abs() < 1e-9), "{near:?}");
    }
}

/// Three sides of a square (tester's report, 2026-11-23): the rows split
/// some of the bevel's own edges at the open ends, and the selection still
/// named the unsplit edges, so the next change was refused. Every edge the
/// bevel selects or calls a strip edge must exist in its result, for every
/// mix of bevel and Mitering, shape and Round Level.
#[test]
fn three_sides_of_a_square_keep_a_valid_selection() {
    let key = |(a, b): (u32, u32)| (a.min(b), a.max(b));
    let cube = primitive::cube_geometry(DVec3::ZERO, DVec3::splat(2.)).unwrap();
    let grid = primitive::segmented_cube(DVec3::ZERO, DVec3::splat(3.), [3, 3, 3]).unwrap();
    let pick = |mesh: &Mesh, keep: &dyn Fn(DVec3) -> bool| -> Vec<(u32, u32)> {
        let v = &mesh.vertices;
        mesh.edges()
            .into_iter()
            .filter(|&(a, b)| keep((v[a as usize] + v[b as usize]) / 2.))
            .map(key)
            .collect()
    };
    // The cube's top square without its back side; the middle square on
    // the 3×3 top without its back side.
    let cube_u = pick(&cube, &|m| m.y > 0.9 && m.z > -0.9);
    let grid_u = pick(&grid, &|m| {
        m.y > 1.4
            && ((m.x.abs() < 0.6 && (m.z - 0.5).abs() < 1e-9)
                || (m.z.abs() < 0.6 && (m.x.abs() - 0.5).abs() < 1e-9))
    });
    assert_eq!((cube_u.len(), grid_u.len()), (3, 3));
    for (name, mesh, edges) in [("cube", &cube, &cube_u), ("grid", &grid, &grid_u)] {
        for rows in [1, 2, 4] {
            for shape in [EdgeBevelShape::Round, EdgeBevelShape::Square] {
                for (width, support) in [(0.1, 0.), (0., 0.1), (0.2, 0.1), (0.05, 0.2), (0.3, 0.3)]
                {
                    let settings = EdgeBevelSettings {
                        width,
                        rows,
                        depth: 100.,
                        chamfer: false,
                        options: EdgeBevelOptions {
                            support,
                            shape,
                            ..Default::default()
                        },
                    };
                    let what = format!("{name} rows {rows} {shape:?} {width}/{support}");
                    let out = edge_bevel_with(mesh, edges, &settings, None)
                        .unwrap_or_else(|e| panic!("{what}: {e}"));
                    out.mesh
                        .validate()
                        .unwrap_or_else(|e| panic!("{what}: {e}"));
                    let all = out.mesh.edges();
                    for e in out.selected_after.iter().chain(&out.strip_edges) {
                        assert!(all.binary_search(e).is_ok(), "{what}: {e:?} isn't an edge");
                    }
                }
            }
        }
    }
}
