//! One store for the tools' last-used values (untangle E1).
//!
//! Each tool remembers the values of its last use (UIC-95): Cone's sizes,
//! Boolean's options, Subdivide's method ... These used to be one `Editor`
//! field per tool, so the editor named every tool's option type. Now each
//! tool file names a key type for its values (`impl PresetKey for
//! ConePreset { type Value = ConeParameters; }`) and the editor keeps one
//! `presets: ToolPresets`, read with `editor.presets.get::<ConePreset>()`.
//!
//! A key that was never written reads as its value type's `Default`, the
//! same value the old field started with, and two stores compare equal
//! when every key reads the same (a written default equals an unwritten
//! key), so `Editor`'s `PartialEq` sees what it saw before. Values are
//! shared between copies of the editor (it is copied for every command)
//! and copied only when one copy writes.
//!
//! Not saved: like the fields it replaces, the store lives for the session.
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

/// A tool's key into [`ToolPresets`]: usually an empty enum in the tool's
/// own file, naming the type of value it keeps.
pub trait PresetKey: 'static {
    type Value: Clone + Default + PartialEq + Send + Sync + 'static;
}

/// A stored value with its type erased.
trait Entry: Send + Sync {
    fn clone_entry(&self) -> Arc<dyn Entry>;
    fn same(&self, other: &dyn Entry) -> bool;
    fn is_default(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Clone + Default + PartialEq + Send + Sync + 'static> Entry for T {
    fn clone_entry(&self) -> Arc<dyn Entry> {
        Arc::new(self.clone())
    }
    fn same(&self, other: &dyn Entry) -> bool {
        other.as_any().downcast_ref::<T>() == Some(self)
    }
    fn is_default(&self) -> bool {
        *self == T::default()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// The tools' last-used values, one per [`PresetKey`].
#[derive(Clone, Default)]
pub struct ToolPresets {
    values: BTreeMap<TypeId, Arc<dyn Entry>>,
}

impl ToolPresets {
    /// The value `K` keeps: its last write, or `K::Value::default()`.
    pub fn get<K: PresetKey>(&self) -> &K::Value {
        match self.values.get(&TypeId::of::<K>()) {
            Some(entry) => downcast::<K>(entry.as_ref()),
            None => default_of::<K>(),
        }
    }

    /// The value `K` keeps, to change in place.
    pub fn get_mut<K: PresetKey>(&mut self) -> &mut K::Value {
        let entry = self
            .values
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Arc::new(K::Value::default()));
        if Arc::get_mut(entry).is_none() {
            // Shared with another copy of the editor: copy it first.
            *entry = entry.clone_entry();
        }
        let entry = Arc::get_mut(entry).expect("a fresh copy is not shared");
        entry
            .as_any_mut()
            .downcast_mut::<K::Value>()
            .expect("a preset key always stores its own value type")
    }

    /// Replaces the value `K` keeps.
    pub fn set<K: PresetKey>(&mut self, value: K::Value) {
        self.values.insert(TypeId::of::<K>(), Arc::new(value));
    }

    /// Forgets `K`'s value, so it reads as the default again (a fresh
    /// launch's, `Command::ResetToolAttributes`).
    pub fn forget<K: PresetKey>(&mut self) {
        self.values.remove(&TypeId::of::<K>());
    }
}

impl PartialEq for ToolPresets {
    /// Equal when every key reads the same; an unwritten key reads as its
    /// default.
    fn eq(&self, other: &Self) -> bool {
        let matches = |a: &Self, b: &Self| {
            a.values.iter().all(|(id, x)| match b.values.get(id) {
                Some(y) => x.same(y.as_ref()),
                None => x.is_default(),
            })
        };
        matches(self, other) && matches(other, self)
    }
}

fn downcast<K: PresetKey>(entry: &dyn Entry) -> &K::Value {
    entry
        .as_any()
        .downcast_ref::<K::Value>()
        .expect("a preset key always stores its own value type")
}

/// `K::Value::default()`, made once per key and kept for the program's
/// life, so `get` can hand out a reference for an unwritten key.
fn default_of<K: PresetKey>() -> &'static K::Value {
    type Defaults = Mutex<HashMap<TypeId, &'static (dyn Any + Send + Sync)>>;
    static DEFAULTS: OnceLock<Defaults> = OnceLock::new();
    let mut defaults = DEFAULTS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let value: &'static (dyn Any + Send + Sync) = *defaults
        .entry(TypeId::of::<K>())
        .or_insert_with(|| Box::leak(Box::new(K::Value::default())));
    value
        .downcast_ref::<K::Value>()
        .expect("a preset key always stores its own value type")
}

#[cfg(test)]
mod tests {
    use super::*;

    enum Size {}
    impl PresetKey for Size {
        type Value = f64;
    }
    enum Name {}
    impl PresetKey for Name {
        type Value = Option<String>;
    }
    /// A second key with the same value type keeps its own value.
    enum OtherSize {}
    impl PresetKey for OtherSize {
        type Value = f64;
    }

    #[test]
    fn an_unwritten_key_reads_as_its_default() {
        let p = ToolPresets::default();
        assert_eq!(*p.get::<Size>(), 0.0);
        assert_eq!(*p.get::<Name>(), None);
    }

    #[test]
    fn writes_are_kept_per_key() {
        let mut p = ToolPresets::default();
        p.set::<Size>(2.0);
        *p.get_mut::<OtherSize>() += 5.0;
        *p.get_mut::<Name>() = Some("cone".into());
        assert_eq!(*p.get::<Size>(), 2.0);
        assert_eq!(*p.get::<OtherSize>(), 5.0);
        assert_eq!(p.get::<Name>().as_deref(), Some("cone"));
        p.forget::<Size>();
        assert_eq!(*p.get::<Size>(), 0.0);
        assert_eq!(*p.get::<OtherSize>(), 5.0);
    }

    #[test]
    fn copies_are_independent() {
        let mut a = ToolPresets::default();
        a.set::<Size>(1.0);
        let mut b = a.clone();
        *b.get_mut::<Size>() = 3.0;
        assert_eq!(*a.get::<Size>(), 1.0);
        assert_eq!(*b.get::<Size>(), 3.0);
    }

    #[test]
    fn a_written_default_equals_an_unwritten_key() {
        let fresh = ToolPresets::default();
        let mut written = ToolPresets::default();
        written.set::<Size>(0.0);
        assert!(fresh == written && written == fresh);
        written.set::<Size>(1.0);
        assert!(fresh != written && written != fresh);
        let mut other = ToolPresets::default();
        other.set::<Size>(1.0);
        assert!(other == written);
        other.set::<Name>(Some(String::new()));
        assert!(other != written);
    }
}
