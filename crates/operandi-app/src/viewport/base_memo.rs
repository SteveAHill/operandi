//! Values computed once per drag from the live tool's base scene (handle
//! frames, rails), and the Local handle frame that uses one.
use super::*;

impl Operandi {
    /// AC-75: the Local handle's frame (on the last selected element's
    /// cluster) for the live Move, Rotate, Scale or Transform tool: the
    /// pivot service's answer, once per drag base.
    pub(crate) fn local_handle_frame(&self) -> Option<operandi_core::ActionFrame> {
        let l = self.editor.live.as_ref()?;
        let context = self.editor.action_context;
        if context.center != CenterMode::Local
            || !matches!(
                l.tool,
                Tool::Move | Tool::Rotate | Tool::Scale | Tool::Transform
            )
        {
            return None;
        }
        let memo = &self.local_handle_memo;
        let stale = matches!(&*memo.0.borrow(), Some((_, (at, _))) if *at != context);
        if stale {
            *memo.0.borrow_mut() = None;
        }
        memo.get(&l.base, || {
            (
                context,
                self.editor
                    .pivot_request(&l.base, l.tool)
                    .local_handle_frame(),
            )
        })
        .1
    }
    pub(crate) fn local_handle_basis(&self) -> Option<glam::DMat3> {
        self.local_handle_frame().map(|f| f.basis)
    }
}

/// A value derived only from a live tool's base scene (the scene before the
/// drag), such as a handle frame. The base stays the same for the whole drag,
/// so the value is computed once instead of on every frame. Keyed on the
/// active item's mesh allocation, its transform and the selection, which is
/// everything these frames read.
pub(crate) struct BaseMemo<T>(std::cell::RefCell<Option<(BaseKey, T)>>);

/// A [`BaseMemo`] whose value also reads the Symmetry setting and the
/// tool's handle rule (ACS-Q-HANDLE1).
pub(crate) type SymmetryMemo<T> = BaseMemo<(HandleMemoKey, T)>;

pub(crate) type HandleMemoKey = (operandi_core::Symmetry, operandi_core::HandlePick);

pub(super) struct BaseKey {
    mesh: Option<operandi_core::Shared<operandi_core::Mesh>>,
    world: Option<glam::DMat4>,
    selection: operandi_core::Selection,
}

impl<T> Default for BaseMemo<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: PartialEq + Clone, T: Clone> BaseMemo<(K, T)> {
    /// `get` for a value that also reads the Symmetry setting (handle
    /// frames pick the primary side, ACS-01), or another key: Symmetry
    /// switched on or off with the tool open recomputes it.
    pub(crate) fn get_with(&self, base: &Scene, symmetry: K, compute: impl FnOnce() -> T) -> T {
        let stale = matches!(&*self.0.borrow(), Some((_, (at, _))) if *at != symmetry);
        if stale {
            *self.0.borrow_mut() = None;
        }
        self.get(base, || (symmetry, compute())).1
    }
}

impl<T: Clone> BaseMemo<T> {
    pub(crate) fn get(&self, base: &Scene, compute: impl FnOnce() -> T) -> T {
        let item = base.active();
        let key = BaseKey {
            mesh: item.map(|i| i.mesh.shared().clone()),
            world: item.and_then(|i| base.world_matrix(i.id).ok()),
            selection: base.selection.clone(),
        };
        if let Some((k, v)) = &*self.0.borrow()
            // The same mesh allocation (a cheap check) means the same mesh.
            && match (&k.mesh, &key.mesh) {
                (Some(a), Some(b)) => operandi_core::Shared::same(a, b),
                (a, b) => a.is_none() && b.is_none(),
            }
            && k.world == key.world
            && k.selection == key.selection
        {
            return v.clone();
        }
        let value = compute();
        *self.0.borrow_mut() = Some((key, value.clone()));
        value
    }
}
