//! Bloom and Glare settings (`RenderSettings::post`): off in new and old
//! scenes, one undo step, save/reopen at native version 68, refused by a
//! version 67 reader, and invalid values refused.
use operandi_core::post_effects::PostEffects;
use operandi_core::*;

fn version(bytes: &[u8]) -> u64 {
    let doc: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    doc["version"].as_u64().unwrap()
}

fn with_bloom_and_glare(e: &Editor) -> render_settings::RenderSettings {
    let mut s = e.scene.render.clone();
    s.post.bloom.enabled = true;
    s.post.bloom.radius = 0.1;
    s.post.bloom.tint = [1.0, 0.5, 0.25];
    s.post.glare.enabled = true;
    s.post.glare.streaks = 6;
    s.post.glare.angle = 15.0;
    s
}

#[test]
fn new_and_old_scenes_start_with_both_off_and_files_leave_them_out() {
    let mut e = Editor::default();
    e.scene.use_new_scene_render_settings();
    assert!(e.scene.render.post.is_default());
    assert!(!e.scene.render.post.is_active());
    assert!(render_settings::RenderSettings::default().post.is_default());
    let bytes = persistence::encode(&e.scene).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("\"post\""));
    assert!(version(&bytes) < 68);
}

#[test]
fn a_change_is_one_undo_step_and_reopens() {
    let mut e = Editor::default();
    let before = e.scene.render.clone();
    let s = with_bloom_and_glare(&e);
    e.dispatch(Command::SetRenderSettings(s.clone())).unwrap();
    assert_eq!(e.scene.render, s);
    let bytes = persistence::encode(&e.scene).unwrap();
    assert_eq!(version(&bytes), 68);
    let back = persistence::decode(&bytes).unwrap();
    assert_eq!(back.render.post, s.post);
    assert_eq!(e.history.undo.last().unwrap().label, "Render Settings");
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(e.scene.render, before);
    e.dispatch(Command::Redo).unwrap();
    assert_eq!(e.scene.render, s);
}

#[test]
fn a_version_67_file_with_them_is_refused() {
    let mut e = Editor::default();
    e.dispatch(Command::SetRenderSettings(with_bloom_and_glare(&e)))
        .unwrap();
    let bytes = persistence::encode(&e.scene).unwrap();
    let mut doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    doc["version"] = 67.into();
    let old = serde_json::to_vec(&doc).unwrap();
    let err = persistence::decode(&old).unwrap_err();
    assert!(err.contains("68"), "{err}");
}

#[test]
fn invalid_values_are_refused_and_change_nothing() {
    let mut e = Editor::default();
    let before = e.scene.render.clone();
    let mut s = before.clone();
    s.post = PostEffects::default();
    s.post.glare.streaks = 5;
    assert!(e.dispatch(Command::SetRenderSettings(s)).is_err());
    let mut s = before.clone();
    s.post.bloom.intensity = -1.0;
    assert!(e.dispatch(Command::SetRenderSettings(s)).is_err());
    assert_eq!(e.scene.render, before);
}
