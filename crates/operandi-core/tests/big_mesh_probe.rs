//! Opt-in probe for very big meshes (hidden-limits B1): a 20 million
//! polygon grid as one mesh, past the old 5 million limit, through
//! validate, an edit, undo, save and reopen, with times and the process's
//! memory printed. Needs about 15 GB of memory; run it in release:
//! `cargo test --release -p operandi-core --test all big_mesh_probe:: -- --ignored --nocapture`
//!
//! 2026-11-22, M-series Mac, release: built 1.1 s (3.4 GB), validate 8.1 s,
//! Flip of all 20M polygons 72 s with 19.9 GB peak (undo holds 3.5 GB),
//! undo instant, save 2.6 s (1.31 GB file), reopen 3.5 s. Edits that
//! rewrite every polygon are the slow part at this size.
use glam::DVec3;
use operandi_core::*;
use std::time::Instant;

fn rss_gb() -> f64 {
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map_or(0., |kb| kb / 1024. / 1024.)
}

#[test]
#[ignore]
fn twenty_million_polygons_in_one_mesh() {
    let side: u32 = 4473; // 4473² ≈ 20.0 million quads
    let start = Instant::now();
    let mut mesh = Mesh::default();
    let row = side + 1;
    mesh.vertices = (0..row * row)
        .map(|i| DVec3::new((i % row) as f64, 0., (i / row) as f64))
        .collect();
    let faces: Vec<Face> = (0..side * side)
        .map(|k| {
            let (x, y) = (k % side, k / side);
            Face {
                vertices: vec![
                    y * row + x,
                    y * row + x + 1,
                    (y + 1) * row + x + 1,
                    (y + 1) * row + x,
                ],
                uv: None,
                material: "Default".into(),
                smooth: false,
                part: None,
                material_label: None,
            }
        })
        .collect();
    mesh.faces = faces.into();
    let polygons = mesh.faces.len();
    assert!(polygons > 19_900_000);
    eprintln!(
        "built {polygons} polygons in {:.1} s, {:.1} GB",
        start.elapsed().as_secs_f64(),
        rss_gb()
    );

    let t = Instant::now();
    mesh.validate().unwrap();
    eprintln!("validate {:.1} s", t.elapsed().as_secs_f64());

    let mut e = Editor::default();
    e.dispatch(Command::NewMesh).unwrap();
    e.scene.active_mut().unwrap().mesh = MeshSlot::new(mesh);
    e.dispatch(Command::Mode(Mode::Polygon)).unwrap();
    e.history.undo.clear();
    let before = e.scene.clone();
    let t = Instant::now();
    e.dispatch(Command::FlipPolygons).unwrap();
    eprintln!(
        "flip all {:.1} s, {:.1} GB, undo holds {:.1} GB",
        t.elapsed().as_secs_f64(),
        rss_gb(),
        e.undo_memory_bytes() as f64 / 1e9
    );
    let t = Instant::now();
    e.dispatch(Command::Undo).unwrap();
    eprintln!("undo {:.1} s", t.elapsed().as_secs_f64());
    assert_eq!(e.scene, before);

    let dir = std::env::temp_dir().join(format!("operandi-big-mesh-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("twenty-million.opscene");
    let t = Instant::now();
    persistence::save(&path, &e.scene).unwrap();
    let size = std::fs::metadata(&path).unwrap().len();
    eprintln!(
        "save {:.1} s, {:.2} GB file",
        t.elapsed().as_secs_f64(),
        size as f64 / 1e9
    );
    let t = Instant::now();
    let reopened = persistence::load(&path).unwrap();
    eprintln!(
        "reopen {:.1} s, {:.1} GB",
        t.elapsed().as_secs_f64(),
        rss_gb()
    );
    assert_eq!(reopened.items[0].mesh.faces.len(), polygons);
    std::fs::remove_dir_all(&dir).unwrap();
}
