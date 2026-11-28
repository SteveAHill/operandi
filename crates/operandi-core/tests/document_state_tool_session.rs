//! Scene tabs and the running tool session (untangle E3 guard): switching
//! tabs takes the open tool, its arming, tweaks and gesture state with the
//! scene, while the bookkeeping of the command running now and the last
//! transform drag stay with the app.
use glam::DVec3;
use operandi_core::*;

fn moving_cube() -> Editor {
    let mut e = Editor::default();
    e.dispatch(Command::NewMesh).unwrap();
    e.dispatch(Command::Activate(Tool::Box)).unwrap();
    e.dispatch(Command::Preview(Parameters {
        size: DVec3::splat(2.),
        ..e.live.as_ref().unwrap().params.clone()
    }))
    .unwrap();
    e.dispatch(Command::Drop).unwrap();
    e.dispatch(Command::Activate(Tool::Move)).unwrap();
    e.dispatch(Command::Preview(Parameters {
        delta: DVec3::X,
        ..e.live.as_ref().unwrap().params.clone()
    }))
    .unwrap();
    e.dispatch(Command::CommitTransformStep).unwrap();
    e
}

#[test]
fn a_tab_switch_takes_the_tool_session_with_the_scene() {
    let mut a = moving_cube();
    assert!(a.live.is_some());
    assert!(a.last_transform_step.is_some());
    a.armed = Some(Tool::Scale);
    a.extend_from_click = true;
    a.gesture_partners = vec![(1, Component::Vertex(0))];
    a.tweak_held = true;
    a.dispatch_depth = 2;
    let before = a.clone();

    let mut b = Editor::default();
    a.swap_document_state(&mut b);
    // The scene's own session went with it.
    assert!(a.live.is_none());
    assert_eq!(a.armed, None);
    assert!(!a.extend_from_click);
    assert!(a.gesture_partners.is_empty());
    assert_eq!(b.live.as_ref().map(|l| l.tool), Some(Tool::Move));
    assert_eq!(b.armed, Some(Tool::Scale));
    assert!(b.extend_from_click);
    assert_eq!(b.gesture_partners, vec![(1, Component::Vertex(0))]);
    // Shared bookkeeping stayed.
    assert!(a.tweak_held);
    assert_eq!(a.dispatch_depth, 2);
    assert!(a.last_transform_step.is_some());
    assert!(!b.tweak_held);
    assert_eq!(b.dispatch_depth, 0);
    assert!(b.last_transform_step.is_none());

    a.swap_document_state(&mut b);
    assert!(a == before);
    assert!(b.live.is_none() && b.armed.is_none() && !b.extend_from_click);
}
