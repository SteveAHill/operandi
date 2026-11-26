//! Edge Bevel's Corner Pattern (a tester's request, 2026-11-24, from Maya
//! pictures of an L-shaped block's inner corner): how the opening where
//! three rounded strips meet is filled. Patch is the measured EBC-J-05
//! grid (the default). Radial and Uniform are grids with their pole at one
//! corner of the opening, on its edge: the rows of the two strips beside
//! the pole run across as rings (Radial) or chevrons (Uniform), and the
//! third strip's rows run into the pole. No point inside the corner joins
//! everything, and every polygon is a quad but the pole's thin triangles.
use crate::edge_bevel_loop_slide::hole;
use glam::DVec3;
use operandi_core::*;

fn cube() -> Mesh {
    primitive::cube_geometry(DVec3::ZERO, DVec3::splat(2.)).unwrap()
}

/// The three edges at the cube's corner (1, 1, 1).
fn cube_corner(m: &Mesh) -> Vec<(u32, u32)> {
    let corner = m
        .vertices
        .iter()
        .position(|p| p.distance(DVec3::ONE) < 1e-9)
        .unwrap() as u32;
    m.edges()
        .into_iter()
        .filter(|&(a, b)| a == corner || b == corner)
        .collect()
}

/// The hole's rim and its four vertical corners: three strips meet at
/// each rim corner, with the inset's diagonal running in too.
const HOLE: [(u32, u32); 8] = [
    (4, 5),
    (5, 6),
    (6, 7),
    (4, 7),
    (4, 8),
    (5, 9),
    (6, 10),
    (7, 11),
];

fn bevel(m: &Mesh, edges: &[(u32, u32)], level: u32, corner: CornerPattern) -> Mesh {
    let settings = EdgeBevelSettings {
        width: 0.1,
        rows: 2 * level,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            corner,
            ..Default::default()
        },
    };
    let out = edge_bevel_with(m, edges, &settings, None)
        .unwrap_or_else(|e| panic!("{corner:?} level {level}: {e}"));
    out.mesh
        .validate()
        .unwrap_or_else(|e| panic!("{corner:?} level {level}: {e}"));
    out.mesh
}

fn sides(m: &Mesh, n: usize) -> usize {
    m.faces.iter().filter(|f| f.vertices.len() == n).count()
}

/// How many polygons each point is used by, at most, among points that
/// aren't on the original mesh's surface edges (the fills' inside points).
fn busiest_point(m: &Mesh) -> usize {
    let mut uses = vec![0usize; m.vertices.len()];
    for f in m.faces.iter() {
        for &v in &f.vertices {
            uses[v as usize] += 1;
        }
    }
    uses.into_iter().max().unwrap_or(0)
}

#[test]
fn radial_and_uniform_are_quad_grids_with_a_pole_on_the_edge() {
    let c = cube();
    // Name, mesh, edges, and how many corners where three strips meet.
    type Shape = (&'static str, Mesh, Vec<(u32, u32)>, usize);
    let shapes: [Shape; 2] = [
        ("cube corner", c.clone(), cube_corner(&c), 1),
        ("hole corners", hole(), HOLE.to_vec(), 4),
    ];
    for (name, m, edges, corners) in shapes {
        for level in [1, 2, 3] {
            let n = 2 * level as usize;
            let patch = bevel(&m, &edges, level, CornerPattern::Patch);
            let radial = bevel(&m, &edges, level, CornerPattern::Radial);
            let uniform = bevel(&m, &edges, level, CornerPattern::Uniform);
            let what = format!("{name}, level {level}");
            for (pattern, mesh) in [("Radial", &radial), ("Uniform", &uniform)] {
                // Each corner: n × n cells, the first row of them triangles;
                // on an inner corner (the hole's) Radial's first row is one
                // polygon of n + 2 sides instead, as in Maya.
                let fan = !(pattern == "Radial" && name == "hole corners");
                if fan {
                    assert_eq!(
                        sides(mesh, 3),
                        sides(&patch, 3) + corners * n,
                        "{what}: {pattern} has n triangles at each pole"
                    );
                } else {
                    assert_eq!(
                        sides(mesh, n + 2) - sides(&patch, n + 2),
                        if n + 2 == 4 {
                            sides(mesh, 4) - sides(&patch, 4)
                        } else {
                            corners
                        },
                        "{what}: {pattern} has one polygon at each pole"
                    );
                }
                // Polygons the bevel cut keep their extra corners; the
                // fill itself adds none of more than four sides (but
                // Radial's top polygons).
                let big = |m: &Mesh| {
                    m.faces
                        .iter()
                        .filter(|f| f.vertices.len() > 4 && (fan || f.vertices.len() != n + 2))
                        .count()
                };
                assert_eq!(
                    big(mesh),
                    big(&patch),
                    "{what}: {pattern} left a polygon of more than four sides"
                );
                // The pole is shared by its n triangles and the polygons
                // around the original corner, never by a whole fan.
                assert!(
                    busiest_point(mesh) <= n + 4,
                    "{what}: {pattern} has a point joining {} polygons",
                    busiest_point(mesh)
                );
            }
            // At Level 1 the only inside point is on the middle line, where
            // a ring and a chevron meet anyway.
            if level >= 2 {
                assert_ne!(
                    radial.vertices, uniform.vertices,
                    "{what}: Radial's rings and Uniform's chevrons differ"
                );
            }
        }
    }
}

/// Mitering Style Default (`Balanced`) is the default (the tester,
/// 2026-11-26; it was Neutral, `NoMitering`).
#[test]
fn default_is_the_default_mitering_style() {
    assert_eq!(EdgeBevelOptions::default().corner, CornerPattern::Balanced);
    assert_eq!(CornerPattern::default(), CornerPattern::Balanced);
}
