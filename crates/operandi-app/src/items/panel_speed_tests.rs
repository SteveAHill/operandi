//! Panels stay quick with a big scene (scorecard row 62). With 10,000
//! items the Item List lays out only the rows on screen, and the
//! Properties panel shows the primary item, so a frame of the default
//! layout draws about the same as with 100 items.
//!
//! Two checks:
//! - `panels_draw_the_same_with_10k_items_as_with_100` counts what a frame
//!   paints (shapes and text), which doesn't depend on the machine, so it
//!   runs with every test pass and fails the day a panel starts drawing
//!   every item.
//! - `panel_frame_time_with_10k_items` times frames. Opt-in, in release:
//!   `cargo test --release -p operandi-app panel_frame_time -- --ignored --nocapture`
use eframe::egui;
use egui::epaint::Shape;

/// An app whose scene has `n` mesh items, all selected (the last one
/// primary), so the Item List and Properties both have the most to show.
fn app_with_items(n: usize) -> crate::Operandi {
    let mut a = crate::Operandi::empty();
    let mut scene = operandi_core::Scene::default();
    for k in 0..n {
        scene.new_mesh_named(Some(format!("Item {k}"))).unwrap();
    }
    let ids: Vec<u64> = scene.items.iter().map(|i| i.id).collect();
    scene.selection.item = ids.last().copied();
    scene.selection.also = ids[..ids.len() - 1].to_vec();
    a.editor.scene = scene;
    a
}

/// Shapes a frame paints, with text counted per glyph row so long lists
/// of labels can't hide inside one shape.
fn painted(output: &egui::FullOutput) -> usize {
    fn count(shape: &Shape) -> usize {
        match shape {
            Shape::Vec(v) => v.iter().map(count).sum(),
            Shape::Text(t) => t.galley.rows.len().max(1),
            _ => 1,
        }
    }
    output.shapes.iter().map(|c| count(&c.shape)).sum()
}

/// A settled frame: the first frames lay out fonts and panel sizes.
fn settled_frame(a: &mut crate::Operandi, ctx: &egui::Context) -> egui::FullOutput {
    let size = egui::vec2(1440., 900.);
    for _ in 0..3 {
        crate::tests::draw(a, ctx, size, Vec::new());
    }
    crate::tests::draw(a, ctx, size, Vec::new())
}

#[test]
fn panels_draw_the_same_with_10k_items_as_with_100() {
    let small = painted(&settled_frame(
        &mut app_with_items(100),
        &egui::Context::default(),
    ));
    let big = painted(&settled_frame(
        &mut app_with_items(10_000),
        &egui::Context::default(),
    ));
    // A little more is fine (wider number columns, a longer scroll bar);
    // drawing every item would be thousands more.
    assert!(
        big <= small + small / 10 + 50,
        "a frame with 10,000 items paints {big} shapes, with 100 items {small}: \
         some panel draws every item instead of what's on screen"
    );
}

/// Frame times with 100 and 10,000 items. Fails when 10,000 items make a
/// frame more than 25 ms slower than 100 do (about two frames at 60 fps,
/// measured best of several, so other work on the machine doesn't count).
#[test]
#[ignore]
fn panel_frame_time_with_10k_items() {
    let best = |n: usize| {
        let mut a = app_with_items(n);
        let ctx = egui::Context::default();
        settled_frame(&mut a, &ctx);
        let frames = std::env::var("FRAMES")
            .ok()
            .and_then(|f| f.parse().ok())
            .unwrap_or(8);
        (0..frames)
            .map(|_| {
                let t = std::time::Instant::now();
                crate::tests::draw(&mut a, &ctx, egui::vec2(1440., 900.), Vec::new());
                t.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min)
    };
    let small = best(100);
    let big = best(10_000);
    eprintln!("best frame: {small:.1} ms with 100 items, {big:.1} ms with 10,000");
    assert!(
        big - small < 25.,
        "10,000 items add {:.1} ms a frame",
        big - small
    );
}
