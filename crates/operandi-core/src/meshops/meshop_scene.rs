//! A throwaway scene for ops whose tool engine reads a scene (Lathe, UV
//! Project, UV Relax): one item at the origin holding the op's input,
//! active, with the polygons the op didn't choose hidden when asked.
use crate::meshop_types::OpContext;
use crate::{Mesh, Mode, Result, Scene};

/// The scene and its one item's id.
pub(crate) fn one_mesh_scene(
    ctx: &OpContext,
    input: &Mesh,
    hide_unchosen: bool,
) -> Result<(Scene, u64)> {
    let mut scene = Scene::default();
    scene.new_mesh()?;
    let id = scene.items.last().map(|i| i.id).ok_or("No mesh")?;
    let item = scene.item_mut(id).ok_or("No mesh")?;
    item.mesh = input.clone().into();
    if hide_unchosen && let Some(chosen) = ctx.selection {
        let on: std::collections::BTreeSet<u32> = chosen.iter().copied().collect();
        item.elements.hidden.faces = (0..input.faces.len() as u32)
            .filter(|f| !on.contains(f))
            .collect();
    }
    scene.select_item(Some(id));
    scene.selection.mode = Mode::Polygon;
    Ok((scene, id))
}
