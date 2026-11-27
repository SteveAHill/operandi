//! A-SEL and C-BORDER (AC-RULES v6): the axes the Selection and Selection
//! Border Action Centers give every tool, and Selection Border's border.
//!
//! **A-SEL.** The frame is the principal axes of the points the tool moves
//! (the eigenvectors of their covariance about their average): X along the
//! largest spread, Y the middle one, Z the least. A flat patch gets Z along
//! its normal, a long strip X along its length. It replaces "Y = the average
//! normal" (2026-11-27), and it never refuses a selection for its shape.
//!
//! **C-BORDER.** The border edges are edges of selected polygons that an
//! unselected polygon uses too; open mesh edges don't count.
use crate::{Mesh, Mode, Selection};
use glam::{DMat3, DMat4, DVec3};
use std::collections::{BTreeMap, BTreeSet};

/// A-SEL "Ties": spreads within this (relative to the largest) are equal.
const TIE: f64 = 1e-6;
/// A component smaller than this counts as zero in the sign rules.
const ZERO: f64 = 1e-9;

/// A-SEL's frame for `points` (columns X, Y, Z; right-handed).
///
/// `normal` is the moved polygons' summed normal, or zero when they have
/// none (wires, or normals that cancel as round a closed band).
///
/// Signs: X has a positive world-X part (if zero, positive world Y, then
/// positive world Z); Z points to the side of `normal`, or follows X's rule
/// without one; Y = Z × X.
///
/// Ties: where two spreads are equal, the higher-ranked axis of the pair
/// takes the direction nearest world X (then world Y, then world Z) made
/// square to the third axis. All three equal, or fewer than two points:
/// World.
pub fn principal_frame(points: &[DVec3], normal: DVec3) -> DMat3 {
    if points.len() < 2 {
        return DMat3::IDENTITY;
    }
    let [(most, e0), (middle, _), (least, e2)] = crate::action_center::principal_axes(points);
    if !(most.is_finite() && most > 1e-24) {
        // Every point in one place.
        return DMat3::IDENTITY;
    }
    let equal = |a: f64, b: f64| (a - b).abs() <= TIE * most;
    if equal(most, least) {
        return DMat3::IDENTITY;
    }
    let (x, z) = if equal(most, middle) {
        // X and Y tie; Z is settled, X goes nearest world X across it.
        (nearest_world_axis(e2), e2)
    } else if equal(middle, least) {
        // Y and Z tie; X is settled, Y goes nearest world X across it.
        (e0, e0.cross(nearest_world_axis(e0)))
    } else {
        (e0, e2)
    };
    let x = positive(x);
    // Squared up against X, so the frame is exactly orthonormal.
    let z = (z - x * z.dot(x)).normalize_or_zero();
    if z == DVec3::ZERO || !z.is_finite() {
        return DMat3::IDENTITY;
    }
    let toward = z.dot(normal);
    let z = if toward.abs() > ZERO {
        z * toward.signum()
    } else {
        positive(z)
    };
    DMat3::from_cols(x, z.cross(x), z)
}

/// `v` or `-v`: the one whose first nonzero world part (X, then Y, then Z)
/// is positive.
fn positive(v: DVec3) -> DVec3 {
    match v.to_array().into_iter().find(|c| c.abs() > ZERO) {
        Some(c) if c < 0. => -v,
        _ => v,
    }
}

/// World X made square to unit `axis`; world Y, then world Z, when the
/// one before runs along `axis`.
fn nearest_world_axis(axis: DVec3) -> DVec3 {
    [DVec3::X, DVec3::Y, DVec3::Z]
        .into_iter()
        .map(|w| w - axis * w.dot(axis))
        .find(|v| v.length() > 1e-6)
        .map_or(DVec3::X, DVec3::normalize)
}

/// A-SEL for a mesh's selection, in world space (`world` is the item's
/// world matrix).
///
/// The points are the ones the tool moves: the selected points, both ends
/// of each selected edge, every corner of each selected polygon; every
/// point with nothing selected. The polygons whose normals are summed for
/// Z's sign are the selected ones, or the ones using the selected edges or
/// points.
///
/// `side` (S-02, Symmetry): only the points it accepts (by their place in
/// the item's own space) count, and only the polygons with such a point.
/// None left gives World.
pub fn selection_axes(
    mesh: &Mesh,
    selection: &Selection,
    world: DMat4,
    side: Option<&dyn Fn(DVec3) -> bool>,
) -> DMat3 {
    let count = mesh.vertices.len();
    let nothing = match selection.mode {
        Mode::Vertex => selection.vertices.is_empty(),
        Mode::Edge => selection.edges.is_empty(),
        Mode::Polygon => selection.faces.is_empty(),
        Mode::Item => true,
    };
    // One flag per point, so each counts once however often it is named.
    let mut moved = vec![false; count];
    let mut mark = |v: u32| {
        if let Some(flag) = moved.get_mut(v as usize) {
            *flag = true;
        }
    };
    let mut polygons: BTreeSet<usize> = BTreeSet::new();
    if nothing {
        (0..count as u32).for_each(&mut mark);
        polygons.extend(0..mesh.faces.len());
    } else {
        match selection.mode {
            Mode::Vertex => {
                let around = mesh.point_faces();
                for &v in &selection.vertices {
                    mark(v);
                    if (v as usize) < count {
                        polygons.extend(around.of(v).iter().map(|&f| f as usize));
                    }
                }
            }
            Mode::Edge => {
                let on_edge = mesh.edge_faces();
                for &(a, b) in &selection.edges {
                    mark(a);
                    mark(b);
                    if let Some(faces) = on_edge.get(&crate::topology::edge_key(a, b)) {
                        polygons.extend(faces.iter().copied());
                    }
                }
            }
            _ => {
                for &f in &selection.faces {
                    if let Some(face) = mesh.faces.get(f as usize) {
                        face.vertices.iter().copied().for_each(&mut mark);
                        polygons.insert(f as usize);
                    }
                }
            }
        }
    }
    if let Some(keep) = side {
        for (v, flag) in moved.iter_mut().enumerate() {
            *flag = *flag && keep(mesh.vertices[v]);
        }
    }
    let points: Vec<DVec3> = moved
        .iter()
        .zip(&mesh.vertices)
        .filter(|(flag, _)| **flag)
        .map(|(_, &p)| world.transform_point3(p))
        .collect();
    // Normals go to the world by the inverse transpose.
    let linear = DMat3::from_mat4(world);
    let to_world = if linear.determinant().abs() > 1e-300 {
        linear.inverse().transpose()
    } else {
        linear
    };
    let mut sum = DVec3::ZERO;
    let mut voters = 0usize;
    for f in polygons {
        if mesh.is_polyline(f)
            || !mesh.faces[f]
                .vertices
                .iter()
                .any(|&v| moved.get(v as usize) == Some(&true))
        {
            continue;
        }
        let n = (to_world * mesh.normal(f)).normalize_or_zero();
        if n.is_finite() && n != DVec3::ZERO {
            sum += n;
            voters += 1;
        }
    }
    // Normals that cancel (a closed band, a whole closed mesh) leave none.
    let normal = if sum.is_finite() && sum.length() > voters as f64 * 1e-10 {
        sum.normalize()
    } else {
        DVec3::ZERO
    };
    principal_frame(&points, normal)
}

/// C-BORDER: the points of the border edges of polygons `faces` of `mesh`:
/// edges of those polygons that at least one other polygon uses too. Open
/// edges (one polygon only) and edges inside the group don't count. Empty
/// when there is no such edge (a whole closed mesh, an open sheet picked
/// whole).
pub fn border_points(mesh: &Mesh, faces: impl IntoIterator<Item = usize>) -> BTreeSet<u32> {
    let chosen: BTreeSet<usize> = faces
        .into_iter()
        .filter(|&f| f < mesh.faces.len() && !mesh.is_polyline(f))
        .collect();
    let mut uses: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for &f in &chosen {
        let face = &mesh.faces[f].vertices;
        for (i, &a) in face.iter().enumerate() {
            let b = face[(i + 1) % face.len()];
            *uses.entry(crate::topology::edge_key(a, b)).or_default() += 1;
        }
    }
    let on_edge = mesh.edge_faces();
    uses.into_iter()
        .filter(|(edge, chosen_uses)| {
            on_edge
                .get(edge)
                .is_some_and(|all| all.len() > *chosen_uses)
        })
        .flat_map(|((a, b), _)| [a, b])
        .collect()
}

/// C-BORDER's points for a selection: `border_points` of the selected
/// polygons. Empty in Edge, Vertex and Item mode, where Selection Border is
/// Selection.
pub fn selection_border_points(mesh: &Mesh, selection: &Selection) -> BTreeSet<u32> {
    if selection.mode != Mode::Polygon {
        return BTreeSet::new();
    }
    border_points(mesh, selection.faces.iter().map(|&f| f as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: DVec3, b: DVec3) {
        assert!((a - b).length() < 1e-9, "{a:?} != {b:?}");
    }

    /// The corners of a box of half sizes `h`, turned by `turn`.
    fn box_points(h: DVec3, turn: glam::DQuat) -> Vec<DVec3> {
        let mut out = Vec::new();
        for x in [-1., 1.] {
            for y in [-1., 1.] {
                for z in [-1., 1.] {
                    out.push(turn * (h * DVec3::new(x, y, z)) + DVec3::new(3., -2., 5.));
                }
            }
        }
        out
    }

    #[test]
    fn x_is_the_largest_spread_and_z_the_least() {
        let turn = glam::DQuat::from_rotation_z(0.3) * glam::DQuat::from_rotation_x(-0.5);
        let f = principal_frame(&box_points(DVec3::new(1., 3., 2.), turn), DVec3::ZERO);
        // The box's own Y is longest, its Z next, its X shortest.
        assert!(f.x_axis.dot(turn * DVec3::Y).abs() > 1. - 1e-9);
        assert!(f.y_axis.dot(turn * DVec3::Z).abs() > 1. - 1e-9);
        assert!(f.z_axis.dot(turn * DVec3::X).abs() > 1. - 1e-9);
        assert!(f.x_axis.x > 0., "X has a positive world-X part");
        assert!(f.z_axis.x > 0., "without a normal Z follows X's rule");
        close(f.y_axis, f.z_axis.cross(f.x_axis));
    }

    #[test]
    fn z_takes_the_side_of_the_summed_normal() {
        let flat: Vec<DVec3> = [(0., 0.), (4., 0.), (4., 1.), (0., 1.)]
            .map(|(x, z)| DVec3::new(x, 0., z))
            .to_vec();
        let up = principal_frame(&flat, DVec3::Y);
        close(up.x_axis, DVec3::X);
        close(up.z_axis, DVec3::Y);
        close(up.y_axis, -DVec3::Z);
        let down = principal_frame(&flat, -DVec3::Y);
        close(down.z_axis, -DVec3::Y);
        close(down.y_axis, DVec3::Z);
    }

    #[test]
    fn ties_take_the_direction_nearest_world_x() {
        // A square turned 45° in the ground plane: X and Y tie.
        let square: Vec<DVec3> = [(1., 0.), (0., 1.), (-1., 0.), (0., -1.)]
            .map(|(x, z)| DVec3::new(x, 0., z))
            .to_vec();
        let f = principal_frame(&square, DVec3::Y);
        close(f.x_axis, DVec3::X);
        close(f.z_axis, DVec3::Y);
        // A square bar along world Z: Y and Z tie, Y goes nearest world X.
        let bar = principal_frame(
            &box_points(DVec3::new(1., 1., 4.), glam::DQuat::IDENTITY),
            DVec3::ZERO,
        );
        close(bar.x_axis, DVec3::Z);
        assert!(bar.y_axis.dot(DVec3::X).abs() > 1. - 1e-9);
        close(bar.y_axis, bar.z_axis.cross(bar.x_axis));
        // A straight line: the same tie.
        let line = principal_frame(&[DVec3::ZERO, DVec3::new(0., 2., 0.)], DVec3::ZERO);
        close(line.x_axis, DVec3::Y);
        assert!(line.y_axis.dot(DVec3::X).abs() > 1. - 1e-9);
    }

    #[test]
    fn nothing_to_go_on_is_world() {
        let cube = box_points(DVec3::ONE, glam::DQuat::from_rotation_y(0.4));
        assert_eq!(principal_frame(&cube, DVec3::Y), DMat3::IDENTITY);
        assert_eq!(principal_frame(&[], DVec3::Y), DMat3::IDENTITY);
        assert_eq!(principal_frame(&[DVec3::ONE], DVec3::Y), DMat3::IDENTITY);
        assert_eq!(
            principal_frame(&[DVec3::ONE, DVec3::ONE], DVec3::Y),
            DMat3::IDENTITY
        );
    }
}
