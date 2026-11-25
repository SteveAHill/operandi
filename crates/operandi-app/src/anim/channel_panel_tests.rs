//! The channel panel (CP-30..54, UIC-188) and its shared value field.
use super::*;
use crate::tests::{box_app, draw};
use operandi_core::animation::ChannelKey;

const SIZE: egui::Vec2 = egui::vec2(1400., 900.);

fn frames(a: &mut Operandi, ctx: &Context, n: usize) {
    for _ in 0..n {
        let _ = draw(a, ctx, SIZE, vec![]);
    }
}

fn panel_rect(ctx: &Context) -> Option<egui::Rect> {
    ctx.memory(|m| m.area_rect(egui::Id::new("channel-panel")))
}

#[test]
fn a_current_set_shows_its_panel_in_the_view_and_none_closes_it() {
    let ctx = Context::default();
    let mut a = box_app();
    a.run(Command::Mode(Mode::Item));
    let id = a.editor.scene.active().unwrap().id;
    frames(&mut a, &ctx, 2);
    a.execute(&format!(
        "operandi.channelSet.create name:Ship channels:{id}:rot.Y,{id}:pos.X"
    ));
    assert!(
        a.editor.scene.current_channel_set().is_some(),
        "{}",
        a.status
    );
    frames(&mut a, &ctx, 3);
    let r = panel_rect(&ctx).expect("the panel is drawn");
    let view = a
        .view_rects
        .get(&a.active_view)
        .copied()
        .unwrap_or(a.viewport_rect);
    assert!(view.contains_rect(r), "{r:?} inside {view:?}");
    let label = crate::channel_panel_ui::row_label(&a.editor.scene, &ChannelKey::new(id, "rot.Y"));
    assert!(label.ends_with(" : rot.Y"), "{label}");
    // Choosing (none) closes it and is no undo step (CP-30, CP-60).
    let steps = a.editor.history.undo.len();
    a.execute("operandi.channelSet.current set:none");
    assert_eq!(a.editor.history.undo.len(), steps);
    let before = ctx.cumulative_pass_nr();
    frames(&mut a, &ctx, 2);
    assert!(ctx.cumulative_pass_nr() > before);
    assert!(a.editor.scene.current_channel_set().is_none());
}

#[test]
fn the_field_shows_units_and_sets_each_kind_with_its_own_command() {
    let mut a = box_app();
    let id = a.editor.scene.active().unwrap().id;
    let s = &a.editor.scene;
    let rot = crate::channel_row::kind(s, &ChannelKey::new(id, "rot.Y"));
    assert_eq!(rot.suffix, "°");
    assert!(rot.rate >= 0.5, "fast enough to scrub");
    let pos = crate::channel_row::kind(s, &ChannelKey::new(id, "pos.X"));
    assert_eq!(pos.suffix, " m");
    assert!(
        crate::channel_row::set_line(s, &ChannelKey::new(id, "pos.X"), 1.5)
            .starts_with("transform.channel ")
    );
    // The line runs as one undo step.
    let steps = a.editor.history.undo.len();
    let line = crate::channel_row::set_line(&a.editor.scene, &ChannelKey::new(id, "pos.X"), 1.5);
    a.execute(&line);
    assert_eq!(a.editor.history.undo.len(), steps + 1, "{}", a.status);
    assert_eq!(
        a.editor.scene.item(id).unwrap().transform.translation.x,
        1.5
    );
}
