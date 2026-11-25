//! The Sculpt tab's Multiresolution section (multires_ui.rs): its buttons
//! are drawn from the form, grey with their reasons (UIC-223) and run the
//! level commands; Attenuate lights up on a level above the base.
use super::tests::draw;
use super::toolbox_form_tests::click_line;
use super::*;
use operandi_core::SculptBrush;

const SIZE: Vec2 = vec2(1440., 1400.);

fn app() -> (Operandi, Context) {
    let mut a = Operandi::empty();
    a.run(Command::NewMesh);
    a.editor.scene.active_mut().unwrap().mesh =
        operandi_core::primitive::cube_geometry(DVec3::ZERO, DVec3::ONE)
            .unwrap()
            .into();
    a.editor.history.undo.clear();
    a.category = toolbox::Category::Sculpt;
    let ctx = Context::default();
    theme::install(&ctx);
    draw(&mut a, &ctx, SIZE, vec![]);
    draw(&mut a, &ctx, SIZE, vec![]);
    (a, ctx)
}

fn grey(a: &Operandi, line: &str) -> Option<String> {
    a.line_state(line).expect("a known command").unavailable
}

fn level(a: &Operandi) -> Option<(u8, u8)> {
    let m = a.editor.scene.active()?.multires.as_ref()?;
    Some((m.current, m.top()))
}

#[test]
fn the_multiresolution_buttons_step_levels_and_grey_with_reasons() {
    let (mut a, ctx) = app();
    let attenuate = crate::sculpt_ui::action(SculptBrush::Attenuate);
    assert!(grey(&a, "operandi.multires.subdivide").is_none());
    for line in [
        "operandi.multires.level step:up",
        "operandi.multires.level step:down",
        "operandi.multires.deleteHigher",
        "operandi.multires.apply",
    ] {
        let why = grey(&a, line).unwrap_or_else(|| panic!("{line} should be greyed"));
        assert!(why.contains("Add Level"), "{line}: {why}");
    }
    assert!(grey(&a, attenuate).is_some());
    click_line(&mut a, &ctx, "operandi.multires.subdivide", Modifiers::NONE);
    click_line(&mut a, &ctx, "operandi.multires.subdivide", Modifiers::NONE);
    assert_eq!(level(&a), Some((2, 2)), "{}", a.status);
    assert!(grey(&a, "operandi.multires.level step:up").is_some());
    assert!(grey(&a, attenuate).is_none());
    click_line(
        &mut a,
        &ctx,
        "operandi.multires.level step:down",
        Modifiers::NONE,
    );
    assert_eq!(level(&a), Some((1, 2)));
    draw(&mut a, &ctx, SIZE, vec![]);
    click_line(
        &mut a,
        &ctx,
        "operandi.multires.deleteHigher",
        Modifiers::NONE,
    );
    assert_eq!(level(&a), Some((1, 1)));
    draw(&mut a, &ctx, SIZE, vec![]);
    click_line(&mut a, &ctx, "operandi.multires.apply", Modifiers::NONE);
    assert_eq!(level(&a), None);
    assert_eq!(a.editor.scene.active().unwrap().mesh.faces.len(), 24);
    // Each click was one undo step.
    a.run(Command::Undo);
    assert_eq!(level(&a), Some((1, 1)));
}
