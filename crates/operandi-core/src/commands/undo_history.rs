//! The undo history an `Editor` owns (untangle E2): undo and redo steps,
//! the marks and counters that tell steps apart, the command records
//! waiting for the next step, and the limits the app sets. Recording and
//! replaying steps needs the scene, so those methods stay on `Editor`
//! (editor/history.rs); this is only the state.
use crate::Checkpoint;

#[derive(Clone, Default, PartialEq)]
pub struct History {
    /// Undo and redo steps, shared: the editor is copied for every command
    /// (identity_editor.rs stages each one), and copying each step's two
    /// scenes made a command in a 100,000-item scene take seconds. Change a
    /// step with `Arc::make_mut` (scene-budget, 2026-11-20).
    pub undo: Vec<std::sync::Arc<Checkpoint>>,
    pub redo: Vec<std::sync::Arc<Checkpoint>>,
    /// While set, undo steps recorded under this gesture id fold into one
    /// (a colour drag across several items and frames is one step). The
    /// app sets it around a gesture's commands and clears it after.
    pub undo_fold: Option<u64>,
    /// Steps recorded since the active tool was activated, for tools that
    /// keep running through Undo/Redo (`mid_tool_undo.rs`). `None` when no
    /// such tool is active or its activation isn't known (re-armed by Undo).
    pub tool_steps: Option<usize>,
    /// Redo depth when the active tool started; leaving the tool drops the
    /// steps undone above it.
    pub tool_redo_floor: Option<usize>,
    /// Counts recorded document steps, to tell a tweak from an Apply.
    pub history_serial: u64,
    /// Counts every change recording makes to the undo history: a new
    /// step, a later edit folded into the newest one, or a selection change
    /// joining a selection group (`recorded_since`). Unlike `undo.len()`, it
    /// still moves once Max Undo Level trims the oldest step.
    pub history_mark: u64,
    /// Selection changes merged into the newest undo step so far; 0 when
    /// that step is not a selection group.
    pub selection_run: u32,
    /// Command records waiting for the next undo step (UIC-226): the app
    /// adds a line before running it, and a tool session's lines (tool on,
    /// `tool.attr`) wait here until its drop. The next recorded step takes
    /// them all; the app drops them when no step came of them.
    pub pending_records: crate::StepRecords,
    /// Undo preferences (Max Undo Level, selection grouping); set by the app.
    pub settings: crate::HistorySettings,
    /// The undo memory budget in bytes; `None` is `UNDO_MEMORY_BUDGET`.
    pub undo_memory_budget: Option<usize>,
    /// Set while a multi-mesh command records one step per mesh that it
    /// then joins into one (multi_tools.rs): trimming in between would
    /// shift the steps it is about to join (Astra P9-001).
    pub undo_trim_paused: bool,
}

impl History {
    /// Whether recording changed the undo history since `mark`: a new step,
    /// an edit folded into the newest one, or a joined selection change.
    pub fn recorded_since(&self, mark: u64) -> bool {
        self.history_mark != mark
    }
    /// How many undo steps are newer than `mark` (still in the history).
    pub fn steps_since(&self, mark: u64) -> usize {
        self.undo.iter().rev().take_while(|c| c.mark > mark).count()
    }
    /// Max Undo Level: drops the oldest steps past it (at least one step
    /// is always allowed).
    pub(crate) fn trim_to_max_undo_level(&mut self) {
        let excess = self
            .undo
            .len()
            .saturating_sub(self.settings.max_undo_level.max(1));
        self.undo.drain(..excess);
    }
}
