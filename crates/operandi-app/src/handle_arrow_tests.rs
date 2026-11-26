//! Handle arrows are drawn in perspective (tester's report, 2026-11-24:
//! "see how long the blue arrow is when pointing close to camera
//! direction"). Each arrow has a fixed length in the scene, 72 px when
//! seen side-on, and shortens as it turns toward the camera instead of
//! being stretched to 72 px along an unstable on-screen direction.
use super::tests::draw;
use super::*;
use crate::camera::View;
use crate::viewport::plane_handles::PLANE_HANDLE_BASE;
use operandi_core::Tool;

/// Move on a 1 m cube at the origin, seen from `view` turned `yaw` degrees
/// about Y (and level).
fn moving(view: View, yaw: f64) -> Operandi {
    let mut a = Operandi::empty();
    a.run(Command::NewMesh);
    a.editor.scene.active_mut().unwrap().mesh =
        operandi_core::cube_geometry(glam::DVec3::ZERO, glam::DVec3::ONE)
            .unwrap()
            .into();
    a.run(Command::Activate(Tool::Move));
    let ctx = Context::default();
    for _ in 0..2 {
        draw(&mut a, &ctx, vec2(1440., 900.), vec![]);
    }
    a.camera.view = view;
    a.camera.target = glam::DVec3::ZERO;
    a.camera.yaw = yaw.to_radians();
    a.camera.pitch = 0.;
    a
}

/// The on-screen length of each axis arrow (X, Y, Z), `None` when hidden.
fn lengths(a: &Operandi) -> [Option<f32>; 3] {
    let handles = a.handles(a.viewport_rect);
    [0, 1, 2].map(|axis| {
        handles
            .iter()
            .find(|h| h.0 == axis)
            .map(|h| h.1.distance(h.2))
    })
}

#[test]
fn arrows_shorten_as_they_turn_toward_the_camera() {
    // Orthographic, level, turned `yaw` from looking down Z: X is seen
    // at cos(yaw) of its length, Z at sin(yaw), Y always side-on.
    for yaw in [0., 10., 30., 45., 60., 80., 90.] {
        let a = moving(View::Orthographic, yaw);
        let [x, y, z] = lengths(&a);
        let rad = f64::to_radians(yaw) as f32;
        for (name, got, want) in [("X", x, 72. * rad.cos()), ("Z", z, 72. * rad.sin())] {
            if want < 7.2 {
                assert_eq!(
                    got, None,
                    "yaw {yaw}: {name} points at the camera, so it hides"
                );
            } else {
                let got = got.unwrap_or_else(|| panic!("yaw {yaw}: {name} is missing"));
                assert!(
                    (got - want).abs() < 1.,
                    "yaw {yaw}: {name} is {got} px, want {want}"
                );
            }
        }
        let y = y.expect("Y is always side-on here");
        assert!((y - 72.).abs() < 1., "yaw {yaw}: Y is {y} px");
    }
}

#[test]
fn an_arrow_nearly_at_the_camera_is_a_short_stub_not_full_length() {
    // The screenshot's case: Perspective, Z a few degrees off the view.
    let a = moving(View::Perspective, 12.);
    let [x, _, z] = lengths(&a);
    let z = z.expect("12° off the view, Z still shows");
    assert!(z < 25., "Z is drawn {z} px; it should be short, not 72");
    let x = x.unwrap();
    assert!(x > 65., "X, nearly side-on, keeps about its length: {x}");
}

#[test]
fn side_on_arrows_keep_their_length_at_any_zoom() {
    for distance in [1.5, 4., 20., 200.] {
        let mut a = moving(View::Perspective, 0.);
        a.camera.distance = distance;
        let [x, y, _] = lengths(&a);
        for (name, got) in [("X", x), ("Y", y)] {
            let got = got.unwrap();
            assert!(
                (got - 72.).abs() < 2.,
                "distance {distance}: {name} is {got} px"
            );
        }
    }
}

#[test]
fn plane_circles_sit_at_the_corner_of_the_shortened_arrows() {
    // Orthographic at 60°: the XY circle's corner is the sum of the drawn
    // X and Y arrows, so it stays with them as X shortens.
    let mut a = moving(View::Orthographic, 60.);
    a.run(Command::Activate(Tool::Transform));
    let r = a.viewport_rect;
    let handles = a.handles(r);
    let tip = |axis: usize| handles.iter().find(|h| h.0 == axis).map(|h| (h.1, h.2));
    let (origin, x_end) = tip(0).expect("X arrow");
    let (_, y_end) = tip(1).expect("Y arrow");
    let xy = PLANE_HANDLE_BASE + 2;
    let Some(circle) = a.plane_ellipse(xy, r) else {
        // Hidden when its plane is too edge-on; at 60° it isn't.
        panic!("XY circle missing");
    };
    let corner = origin + (x_end - origin) + (y_end - origin);
    assert!(
        circle.center.distance(corner) < 1.,
        "circle at {:?}, arrows' corner at {corner:?}",
        circle.center
    );
}
