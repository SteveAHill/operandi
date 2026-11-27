//! Pie menus as forms (one section of `COMMANDS`, in table order): opening
//! a pie by id, editing its slots in the Form Editor, Assign Key, Export
//! Pie and Import Pie (UIC-308..313). Operandi-only names (UIC-165).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- pies
    // UIC-311: the Input Map binds this line, so any pie can have a key.
    script("operandi.pie.open", View, "Opens the pie id: (a form with style pie, e.g. pie.view) centred on the pointer. From a key it opens while the key is held; typed, it waits for a click (UIC-311).")
        .operandi()
        .args(&[arg("id", TEXT, N)])
        .example("operandi.pie.open pie.view")
        .ui(),
    // UIC-311: Assign Key opens the Input Map on the pie's line; with
    // key: it puts that key there at once, with the usual clash warning.
    script("operandi.pie.assignKey", View, "Opens the Input Map on pie id:'s line, waiting for its key; key: puts that key there straight away (with the usual clash warning; nothing is saved until Save Input Map).")
        .operandi()
        .args(&[arg("id", TEXT, N), opt("key", TEXT, N, None)])
        .example("operandi.pie.assignKey pie.view key:Ctrl+Space")
        .ui(),
    // UIC-312: one file, the pie and every sub-pie it opens.
    script("operandi.pie.export", View, "Export Pie: writes pie id: and every sub-pie it opens to one file, filename: (else the data folder's Saved Pies folder).")
        .operandi()
        .args(&[arg("id", TEXT, N), opt("filename", ArgType::Path, N, None)])
        .example("operandi.pie.export pie.view")
        .side_effect(),
    script("operandi.pie.import", View, "Import Pie: reads filename: (a file written by Export Pie; bare, it asks for one). A pie whose id is taken asks first; onClash:rename keeps both (the new one gets a free id), onClash:replace overwrites yours.")
        .operandi()
        .args(&[opt("filename", ArgType::Path, N, None), opt("onClash", ArgType::Choice(&["ask", "rename", "replace"]), N, Some("ask"))])
        .example("operandi.pie.import {~/View.oppie} onClash:rename")
        .ui(),
    // The Form Editor's pie slots (UIC-309, UIC-310): a pie has eight
    // places; these keep every other slot where it is.
    script("operandi.form.slot", View, "Sets slot index: (0 = top, clockwise) of pie form: to run line:, to open sub-pie sub:, or, with neither, to be empty. label: names it. A pie can't open itself, even through its sub-pies (UIC-310).")
        .operandi()
        .args(&[arg("index", INT, N), opt("line", TEXT, N, None), opt("sub", TEXT, N, None), opt("label", TEXT, N, None), opt("form", TEXT, N, None)])
        .example("operandi.form.slot index:2 line:{View Right} label:Right form:my.pie")
        .ui(),
    script("operandi.form.swap", View, "Swaps slots a: and b: of pie form: (the Form Editor's drag between slots); the other slots stay where they are.")
        .operandi()
        .args(&[arg("a", INT, N), arg("b", INT, N), opt("form", TEXT, N, None)])
        .example("operandi.form.swap a:0 b:4 form:my.pie")
        .ui(),
];
