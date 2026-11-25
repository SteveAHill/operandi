//! Right-drag Sides and Segments on a selected Mesh Stack primitive op
//! (GitHub #26), as the create tools do (UIC-76): with the op's row
//! selected and no tool on, sideways is Sides (the Cube: Segments X), up is
//! Segments (the Cube: Segments Z); the drag is one undo step.
use super::tests::draw;
use super::*;
use operandi_core::haul::{self, feel};

const SIZE: Vec2 = vec2(1440., 900.);

fn press(at: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: at,
        pressed,
        modifiers: Modifiers::NONE,
        button: PointerButton::Secondary,
    }
}

fn right_drag(a: &mut Operandi, ctx: &Context, from: Pos2, to: Pos2) {
    draw(a, ctx, SIZE, vec![Event::PointerMoved(from)]);
    draw(a, ctx, SIZE, vec![press(from, true)]);
    for k in 1..=8 {
        let at = from + (to - from) * (k as f32 / 8.);
        draw(a, ctx, SIZE, vec![Event::PointerMoved(at)]);
    }
    draw(a, ctx, SIZE, vec![press(to, false)]);
}

fn app(ctx: &Context, op_type: &str) -> Operandi {
    theme::install(ctx);
    let mut a = Operandi::empty();
    a.run(Command::NewMesh);
    a.run(Command::MeshOpCreate(op_type.into()));
    for _ in 0..2 {
        draw(&mut a, ctx, SIZE, vec![]);
    }
    a
}

fn value(a: &Operandi, key: &str) -> Option<u64> {
    a.selected_primitive_op()?.1.get(key)?.as_u64()
}

/// Off the handles: a corner of the view, away from the shape's centre.
fn away(a: &Operandi) -> Pos2 {
    a.viewport_rect.center() + vec2(300., 220.)
}

#[test]
fn right_drag_changes_a_primitive_ops_counts_in_whole_steps() {
    let ctx = Context::default();
    let px = feel::WHOLE_STEP_PX;
    for row in haul::meshop_table() {
        let mut a = app(&ctx, &row.tool);
        let from = away(&a);
        let sideways = row.right.sideways.clone().unwrap();
        let before = value(&a, &sideways).expect("the op has the value");
        let history = a.editor.history.undo.len();
        right_drag(&mut a, &ctx, from, from + vec2(2. * px + 3., 0.));
        assert_eq!(
            value(&a, &sideways),
            Some(before + 2),
            "{}: {}",
            row.tool,
            a.status
        );
        assert_eq!(
            a.editor.history.undo.len(),
            history + 1,
            "{}: one step",
            row.tool
        );
        a.run(Command::Undo);
        assert_eq!(value(&a, &sideways), Some(before), "{}: undo", row.tool);
        if let Some(up) = row.right.up.clone() {
            let before = value(&a, &up).unwrap();
            right_drag(&mut a, &ctx, from, from - vec2(0., px + 3.));
            assert_eq!(
                value(&a, &up),
                Some(before + 1),
                "{}: {}",
                row.tool,
                a.status
            );
        }
    }
}

#[test]
fn counts_stop_at_the_ops_own_minimum() {
    let ctx = Context::default();
    let mut a = app(&ctx, "prim.cylinder.item");
    let from = away(&a);
    // Far to the left: Sides goes down, never below what the op takes.
    right_drag(&mut a, &ctx, from, from - vec2(600., 0.));
    let sides = value(&a, "sides").unwrap();
    assert!((1..24).contains(&sides), "{sides}: {}", a.status);
    assert!(a.editor.scene.items[0].mesh_stack.is_some());
}

#[test]
fn a_live_tool_or_another_op_keeps_right_drag() {
    let ctx = Context::default();
    // A tool on: the op's row doesn't claim the drag.
    let mut a = app(&ctx, "prim.cylinder.item");
    a.activate(Tool::Box);
    let before = a.editor.scene.items[0].mesh_stack.clone();
    let from = away(&a);
    right_drag(&mut a, &ctx, from, from + vec2(60., 0.));
    assert_eq!(a.editor.scene.items[0].mesh_stack, before);
    // An op with no row (Triangulate) leaves right-drag to the view.
    let b = app(&ctx, "pmodel.triangulate.item");
    assert!(b.haul_owner(haul::Button::Right).is_none());
}
