//! Multiresolution sculpting (multires.rs, provisional MR-Q01..MR-Q08):
//! add, step, delete and apply levels; detail carried across levels;
//! undo, redo and save/reopen; stale polygons; Attenuate on a level.
use glam::DVec3;
use operandi_core::multires::STALE;
use operandi_core::sculpt::{Dab, Stroke, StrokeSetup};
use operandi_core::*;

fn run(e: &mut Editor, line: &str) -> Result<Option<String>> {
    command_line::run_line(e, line).expect("a headless command line")
}

/// A unit cube, active, in Item mode.
fn cube() -> (Editor, u64) {
    let mut e = Editor::default();
    e.dispatch(Command::NewMesh).unwrap();
    e.scene.active_mut().unwrap().mesh = primitive::cube_geometry(DVec3::ZERO, DVec3::ONE)
        .unwrap()
        .into();
    let id = e.scene.active().unwrap().id;
    (e, id)
}

fn mesh(e: &Editor, id: u64) -> Mesh {
    Mesh::clone(&e.scene.item(id).unwrap().mesh)
}

fn level(e: &Editor, id: u64) -> Option<(u8, u8)> {
    e.scene
        .item(id)
        .unwrap()
        .multires
        .as_ref()
        .map(|m| (m.current, m.top()))
}

/// Moves point `v` of the mesh by `d` as one sculpt stroke.
fn nudge(e: &mut Editor, id: u64, v: u32, d: DVec3) {
    let p = e.scene.item(id).unwrap().mesh.vertices[v as usize] + d;
    e.dispatch(Command::SculptStroke(Box::new(SculptEdit {
        label: "Push Stroke".into(),
        items: vec![SculptItemEdit {
            item: id,
            points: vec![(v, p)],
        }],
    })))
    .unwrap();
}

fn reopen(e: &Editor) -> Scene {
    persistence::decode(&persistence::encode(&e.scene).unwrap()).unwrap()
}

#[test]
fn add_level_subdivides_and_undo_restores_the_plain_mesh() {
    let (mut e, id) = cube();
    let plain = mesh(&e, id);
    run(&mut e, "operandi.multires.subdivide").unwrap();
    assert_eq!(level(&e, id), Some((1, 1)));
    assert_eq!(mesh(&e, id).faces.len(), 24);
    assert_eq!(mesh(&e, id).vertices.len(), 26);
    run(&mut e, "operandi.multires.subdivide").unwrap();
    assert_eq!(level(&e, id), Some((2, 2)));
    assert_eq!(mesh(&e, id).faces.len(), 96);
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(level(&e, id), Some((1, 1)));
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(level(&e, id), None);
    assert_eq!(mesh(&e, id), plain);
    e.dispatch(Command::Redo).unwrap();
    e.dispatch(Command::Redo).unwrap();
    assert_eq!(level(&e, id), Some((2, 2)));
}

#[test]
fn stepping_down_and_up_returns_the_same_meshes() {
    let (mut e, id) = cube();
    let plain = mesh(&e, id);
    run(&mut e, "operandi.multires.subdivide").unwrap();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    nudge(&mut e, id, 40, DVec3::new(0.01, 0.02, 0.05));
    let top = mesh(&e, id);
    run(&mut e, "operandi.multires.level step:down").unwrap();
    assert_eq!(level(&e, id), Some((1, 2)));
    run(&mut e, "operandi.multires.level level:0").unwrap();
    assert_eq!(mesh(&e, id), plain);
    run(&mut e, "operandi.multires.level level:2").unwrap();
    let back = mesh(&e, id);
    assert_eq!(back.faces, top.faces);
    for (a, b) in back.vertices.iter().zip(&top.vertices) {
        assert!(a.distance(*b) < 1e-12, "{a} vs {b}");
    }
    // Each step is one undo step.
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(level(&e, id), Some((0, 2)));
}

#[test]
fn detail_rides_along_when_the_base_moves() {
    let (mut e, id) = cube();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    // A bump out of the +X face at level 2, along that face's normal.
    let m = mesh(&e, id);
    let v = (0..m.vertices.len())
        .max_by(|&a, &b| m.vertices[a].x.total_cmp(&m.vertices[b].x))
        .unwrap() as u32;
    let before = m.vertices[v as usize];
    nudge(&mut e, id, v, DVec3::X * 0.1);
    let bump = mesh(&e, id).vertices[v as usize] - before;
    // Down to the base, push the whole +X side out by 0.5 m, back up.
    run(&mut e, "operandi.multires.level level:0").unwrap();
    let base = mesh(&e, id);
    let mut edit = Vec::new();
    for (i, p) in base.vertices.iter().enumerate() {
        if p.x > 0. {
            edit.push((i as u32, *p + DVec3::X * 0.5));
        }
    }
    e.dispatch(Command::SculptStroke(Box::new(SculptEdit {
        label: "Move".into(),
        items: vec![SculptItemEdit {
            item: id,
            points: edit,
        }],
    })))
    .unwrap();
    run(&mut e, "operandi.multires.level level:2").unwrap();
    let after = mesh(&e, id);
    // Without the detail, where the point would sit now.
    let smooth = {
        let mut f = e.clone();
        run(&mut f, "operandi.multires.level level:0").unwrap();
        let s = f.scene.item(id).unwrap().multires.clone().unwrap();
        let mut plain = (*s).clone();
        for l in &mut plain.levels {
            l.detail.iter_mut().for_each(|d| *d = DVec3::ZERO);
        }
        f.scene.item_mut(id).unwrap().multires = Some(Shared::new(plain));
        run(&mut f, "operandi.multires.level level:2").unwrap();
        mesh(&f, id).vertices[v as usize]
    };
    let carried = after.vertices[v as usize] - smooth;
    assert!(
        (carried.length() - bump.length()).abs() < 1e-9,
        "{carried} vs {bump}"
    );
    // The face kept facing +X, so the bump still points out along it.
    assert!(carried.normalize().dot(DVec3::X) > 0.99, "{carried}");
}

#[test]
fn multires_saves_and_reopens() {
    let (mut e, id) = cube();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    nudge(&mut e, id, 30, DVec3::new(0., 0., 0.07));
    run(&mut e, "operandi.multires.level step:down").unwrap();
    let back = reopen(&e);
    assert_eq!(
        back.item(id).unwrap().multires,
        e.scene.item(id).unwrap().multires
    );
    assert_eq!(Mesh::clone(&back.item(id).unwrap().mesh), mesh(&e, id));
    let mut f = Editor {
        scene: back,
        ..Default::default()
    };
    run(&mut f, "operandi.multires.level step:up").unwrap();
    run(&mut e, "operandi.multires.level step:up").unwrap();
    assert_eq!(mesh(&f, id), mesh(&e, id));
    // The file says version 64; a plain scene keeps its older number.
    let text = String::from_utf8(persistence::encode(&e.scene).unwrap()).unwrap();
    assert!(text.contains("\"version\": 64"), "{}", &text[..200]);
    let (plain, _) = cube();
    let text = String::from_utf8(persistence::encode(&plain.scene).unwrap()).unwrap();
    assert!(!text.contains("\"version\": 64"));
}

#[test]
fn damaged_detail_is_refused_on_load() {
    let (mut e, id) = cube();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    let mut s = e.scene.clone();
    let mut m = (**s.item(id).unwrap().multires.as_ref().unwrap()).clone();
    m.levels[0].detail.pop();
    s.item_mut(id).unwrap().multires = Some(Shared::new(m));
    assert!(s.validate().is_err());
    let mut m = (**e.scene.item(id).unwrap().multires.as_ref().unwrap()).clone();
    m.current = 3;
    s.item_mut(id).unwrap().multires = Some(Shared::new(m));
    assert!(s.validate().is_err());
}

#[test]
fn changed_polygons_stop_stepping_until_applied() {
    let (mut e, id) = cube();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    // A tool that rebuilds the polygons (here: a fresh cube in its place).
    let other = primitive::cube_geometry(DVec3::ZERO, DVec3::ONE).unwrap();
    let mut s = e.scene.clone();
    s.item_mut(id).unwrap().mesh = other.clone().into();
    e.scene = s;
    let err = run(&mut e, "operandi.multires.level level:0").unwrap_err();
    assert_eq!(err, STALE);
    assert_eq!(
        e.multires_unavailable(multires::LevelChange::Down, None)
            .as_deref(),
        Some(STALE)
    );
    run(&mut e, "operandi.multires.apply").unwrap();
    assert_eq!(level(&e, id), None);
    assert_eq!(mesh(&e, id), other);
}

#[test]
fn delete_higher_and_apply() {
    let (mut e, id) = cube();
    for _ in 0..3 {
        run(&mut e, "operandi.multires.subdivide").unwrap();
    }
    run(&mut e, "operandi.multires.level level:1").unwrap();
    run(&mut e, "operandi.multires.deleteHigher").unwrap();
    assert_eq!(level(&e, id), Some((1, 1)));
    assert!(run(&mut e, "operandi.multires.deleteHigher").is_err());
    assert!(run(&mut e, "operandi.multires.level step:up").is_err());
    run(&mut e, "operandi.multires.apply").unwrap();
    assert_eq!(level(&e, id), None);
    assert_eq!(mesh(&e, id).faces.len(), 24);
    // At the base, deleting every level leaves a plain mesh again.
    let (mut e, id) = cube();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    run(&mut e, "operandi.multires.level step:down").unwrap();
    run(&mut e, "operandi.multires.deleteHigher").unwrap();
    assert_eq!(level(&e, id), None);
}

#[test]
fn refusals_name_their_reason() {
    let mut e = Editor::default();
    assert!(run(&mut e, "operandi.multires.subdivide").is_err());
    e.dispatch(Command::NewMesh).unwrap();
    let err = run(&mut e, "operandi.multires.subdivide").unwrap_err();
    assert!(err.contains("no polygons"), "{err}");
    let (mut e, _) = cube();
    let err = run(&mut e, "operandi.multires.level step:up").unwrap_err();
    assert!(err.contains("Add Level"), "{err}");
    assert!(run(&mut e, "operandi.multires.level step:sideways").is_err());
    assert!(run(&mut e, "operandi.multires.level").is_err());
    for _ in 0..multires::MAX_LEVELS {
        if run(&mut e, "operandi.multires.subdivide").is_err() {
            break;
        }
    }
    // The polygon budget or the level cap stops it with a reason.
    let steps = e.history.undo.len();
    assert!(run(&mut e, "operandi.multires.subdivide").is_err());
    assert_eq!(e.history.undo.len(), steps);
}

#[test]
fn query_reports_levels() {
    let (mut e, id) = cube();
    let plain = run(&mut e, "operandi.multires.query").unwrap().unwrap();
    assert!(plain.contains("\"multiresolution\":false"), "{plain}");
    run(&mut e, "operandi.multires.subdivide").unwrap();
    let text = run(&mut e, &format!("operandi.multires.query item:{id}"))
        .unwrap()
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["level"], 1);
    assert_eq!(v["levels"], 1);
    assert_eq!(v["polygons"], serde_json::json!([6, 24]));
    assert_eq!(v["in_step"], true);
}

fn setup(brush: SculptBrush) -> StrokeSetup {
    StrokeSetup {
        brush,
        settings: brush.defaults(),
        ctrl: false,
        shift: false,
        symmetry: Symmetry::OFF,
        falloff: None,
        lock_borders: false,
        lock_corners: false,
        radius: 0.6,
        weight_map: None,
    }
}

#[test]
fn attenuate_fades_a_levels_detail() {
    let (mut e, id) = cube();
    assert!(
        SculptBrush::Attenuate
            .unavailable_on(&e.scene, &[id])
            .is_some()
    );
    assert!(Stroke::begin(&e.scene, &[id], setup(SculptBrush::Attenuate)).is_err());
    run(&mut e, "operandi.multires.subdivide").unwrap();
    run(&mut e, "operandi.multires.subdivide").unwrap();
    assert!(
        SculptBrush::Attenuate
            .unavailable_on(&e.scene, &[id])
            .is_none()
    );
    // Lift the top face's centre point.
    let m = mesh(&e, id);
    let v = (0..m.vertices.len())
        .min_by(|&a, &b| {
            let d = |i: usize| m.vertices[i].distance(DVec3::new(0., 0., 0.5));
            d(a).total_cmp(&d(b))
        })
        .unwrap();
    let rest = m.vertices[v];
    nudge(&mut e, id, v as u32, DVec3::Z * 0.2);
    let mut scene = e.scene.clone();
    let mut s = Stroke::begin(&scene, &[id], setup(SculptBrush::Attenuate))
        .unwrap()
        .unwrap();
    for _ in 0..30 {
        s.dab(Dab {
            center: rest + DVec3::Z * 0.2,
            radius: 0.6,
            view: DVec3::Z,
            normal: DVec3::Z,
            pressure: 1.,
        });
    }
    s.flush(&mut scene);
    let p = scene.item(id).unwrap().mesh.vertices[v];
    assert!(
        p.distance(rest) < 0.2 * 0.5,
        "Attenuate pulls the bump back toward the level's rest shape: {p} vs {rest}"
    );
    // Back at the base, there's nothing to fade.
    run(&mut e, "operandi.multires.level level:0").unwrap();
    assert!(
        SculptBrush::Attenuate
            .unavailable_on(&e.scene, &[id])
            .is_some()
    );
}

/// Speed probe: Add Level, stepping and an Attenuate stroke's setup on a
/// 1.5M-polygon level. Run with
/// `cargo test --release --locked --workspace -- multires::speed_probe --ignored --nocapture`.
#[test]
#[ignore]
fn speed_probe() {
    let (mut e, id) = cube();
    for _ in 0..5 {
        run(&mut e, "operandi.multires.subdivide").unwrap();
    }
    run(&mut e, "operandi.multires.apply").unwrap();
    for _ in 0..3 {
        run(&mut e, "operandi.multires.subdivide").unwrap();
    }
    let time = |e: &mut Editor, line: &str| {
        let t = std::time::Instant::now();
        run(e, line).unwrap();
        let polys = e.scene.item(id).unwrap().mesh.faces.len();
        eprintln!("{line}: {:?} ({polys} polygons)", t.elapsed());
    };
    time(&mut e, "operandi.multires.subdivide");
    time(&mut e, "operandi.multires.level step:down");
    time(&mut e, "operandi.multires.level step:up");
    time(&mut e, "operandi.multires.level level:0");
    time(&mut e, "operandi.multires.level level:4");
    let t = std::time::Instant::now();
    let s = Stroke::begin(&e.scene, &[id], setup(SculptBrush::Attenuate)).unwrap();
    eprintln!("Attenuate press: {:?}", t.elapsed());
    assert!(s.is_some());
}
