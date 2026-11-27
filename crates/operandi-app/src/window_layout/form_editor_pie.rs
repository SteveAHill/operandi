//! The Form Editor's pie editing (UIC-308..313): a ring-shaped preview of
//! the selected pie, its eight slots placed as the pie opens them. Click a
//! slot to edit it below (command line typed or from the Command History,
//! label, icon, sub-pie, or empty); drag a slot onto another to swap them.
//! Assign Key opens the Input Map on the pie's line (UIC-311), Export Pie
//! writes it with its sub-pies (UIC-312). Every edit is an
//! `operandi.form.*` or `operandi.pie.*` line (UIC-160, UIC-220).
use super::*;
use crate::pie_menu::{self, SLOTS};

/// The ring preview's size at full scale; narrower panels scale it down.
const RING: Vec2 = vec2(440., 250.);
/// Where each slot sits, in words, clockwise from the top.
const PLACES: [&str; SLOTS] = [
    "Top",
    "Top Right",
    "Right",
    "Bottom Right",
    "Bottom",
    "Bottom Left",
    "Left",
    "Top Left",
];

/// What a pie slot holds, for its editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Command,
    SubPie,
    Empty,
}

fn kind_of(c: Option<&Control>) -> Kind {
    match c {
        Some(c) if c.command.is_some() => Kind::Command,
        Some(c) if c.form.is_some() => Kind::SubPie,
        _ => Kind::Empty,
    }
}

impl Operandi {
    /// The selected pie's ring preview and slot editor, in place of the
    /// add-control rows other forms have.
    pub(super) fn pie_form_editor(&mut self, ui: &mut Ui, form: &Form) {
        let id = form.id.clone();
        let selected = match &self.form_editor.selected {
            Some((f, Some(i))) if *f == id && *i < SLOTS => Some(*i),
            _ => None,
        };
        // UIC-311: the pie's key, and Assign Key.
        let line = pie_menu::line(&id);
        let key = self
            .bindings
            .get(&line)
            .filter(|k| !k.is_empty())
            .map(|k| input::display_chord(k))
            .unwrap_or_else(|| "no key".into());
        ui.horizontal(|ui| {
            ui.add_sized([96., 18.], egui::Label::new("Key"));
            ui.label(&key);
            if ui
                .button("Assign Key…")
                .on_hover_text("Opens the Input Map on this pie's row: press the keys you want")
                .clicked()
            {
                self.execute(&format!("operandi.pie.assignKey {id}"));
            }
        });
        ui.add_space(4.);
        self.pie_ring(ui, form, selected);
        ui.add_space(4.);
        match selected {
            Some(i) => self.pie_slot_editor(ui, form, i),
            None => {
                ui.weak("Click a slot to edit it; drag one onto another to swap them.");
            }
        }
        ui.add_space(4.);
        ui.horizontal(|ui| {
            if ui
                .button("Export Pie…")
                .on_hover_text("Saves this pie and every sub-pie it opens to one file")
                .clicked()
            {
                self.execute(&format!("operandi.pie.export {id}"));
            }
            if ui
                .button("Open Pie")
                .on_hover_text("Opens the pie at the pointer, as its key does")
                .clicked()
            {
                self.execute(&format!("operandi.pie.open {id}"));
            }
        });
    }

    /// Where slot `i` of the ring preview sits, in `area` scaled by `s`.
    fn ring_rect(area: Rect, i: usize, s: f32) -> Rect {
        let r = pie_menu::label_rect(Pos2::ZERO, i, 120.);
        Rect::from_min_max(
            area.center() + r.min.to_vec2() * s,
            area.center() + r.max.to_vec2() * s,
        )
    }

    /// The ring preview: the pie's name in the centre and its slots around
    /// it (UIC-313); click selects, drag onto another swaps.
    fn pie_ring(&mut self, ui: &mut Ui, form: &Form, selected: Option<usize>) {
        let id = form.id.clone();
        let width = ui.available_width().max(200.);
        let s = (width / RING.x).min(1.);
        let (area, _) = ui.allocate_exact_size(RING * s, Sense::hover());
        let pie = crate::pie_forms::pie_from_form(&self.forms, form);
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 6., theme::chrome::background());
        // The name plate in the centre.
        let font = FontId::proportional(12. * s.max(0.75));
        let name = painter.layout(pie.title.clone(), font.clone(), theme::text(), 130. * s);
        let plate = Rect::from_center_size(area.center(), name.size() + vec2(12., 6.));
        painter.rect(
            plate,
            plate.height() / 2.,
            theme::popup_fill(),
            Stroke::new(1.0f32, theme::border()),
            StrokeKind::Inside,
        );
        painter.galley(area.center() - name.size() / 2., name, theme::text());
        let mut hovered = None;
        let mut released_on = None;
        for (i, place) in PLACES.iter().enumerate() {
            let r = Self::ring_rect(area, i, s);
            let response = ui.interact(
                r,
                ui.id().with(("pie-slot", &id, i)),
                Sense::click_and_drag(),
            );
            self.form_editor.rects.insert(format!("{id}@{i}"), r);
            if response.hovered() {
                hovered = Some(i);
            }
            if response.clicked() {
                self.form_editor.selected = Some((id.clone(), Some(i)));
            }
            if response.drag_started() {
                self.form_editor.pie_drag = Some((id.clone(), i));
            }
            if ui.input(|inp| inp.pointer.any_released())
                && ui.input(|inp| inp.pointer.interact_pos().is_some_and(|p| r.contains(p)))
            {
                released_on = Some(i);
            }
            let entry = pie.slot(i);
            let status = entry.map(|e| self.slot_status(e, std::iter::once(id.as_str())));
            let lit = selected == Some(i);
            let dragging_here = self
                .form_editor
                .pie_drag
                .as_ref()
                .is_some_and(|(f, from)| *f == id && *from != i && hovered == Some(i));
            let (fill, text) = match status {
                None => (theme::chrome::background(), theme::weak_text()),
                Some(st) if st.unknown => (theme::popup_fill(), theme::error()),
                Some(st) if !st.enabled => (theme::popup_fill(), theme::disabled_text()),
                Some(_) => (theme::popup_fill(), theme::text()),
            };
            let stroke = if lit || dragging_here {
                Stroke::new(2.0f32, theme::orange())
            } else if status.is_some_and(|st| st.unknown) {
                Stroke::new(1.0f32, theme::error())
            } else {
                Stroke::new(1.0f32, theme::border())
            };
            painter.rect(r, 5. * s, fill, stroke, StrokeKind::Inside);
            let label = match entry {
                Some(e) if e.sub.is_some() => format!("{} ▸", e.label),
                Some(e) => e.label.clone(),
                None => "Empty".to_owned(),
            };
            let galley = painter.layout(label, font.clone(), text, r.width() - 8.);
            let at = r.center() - galley.size() / 2.;
            painter.galley(at, galley, text);
            if let Some(icon) = pie.shows(i).0 {
                let size = (r.height() - 8.).min(16.);
                let ir = Rect::from_min_size(
                    pos2(r.left() + 4., r.center().y - size / 2.),
                    Vec2::splat(size),
                );
                icons::paint(ui, ir, icon);
            }
            let hint = match entry {
                Some(e) if e.sub.is_some() => format!("{place}: opens {}", e.command),
                Some(e) => format!("{place}: {}", e.command),
                None => format!("{place}: empty"),
            };
            response.on_hover_text(hint);
        }
        if let Some(to) = released_on
            && let Some((f, from)) = self.form_editor.pie_drag.take()
            && f == id
            && from != to
        {
            self.execute(&format!("operandi.form.swap a:{from} b:{to} form:{id}"));
        }
        if ui.input(|i| !i.pointer.any_down()) {
            self.form_editor.pie_drag = None;
        }
    }

    /// Slot `i`'s editor: what it holds and its label and icon.
    fn pie_slot_editor(&mut self, ui: &mut Ui, form: &Form, i: usize) {
        let id = form.id.clone();
        let control = form.controls.get(i).cloned();
        let kind = kind_of(control.as_ref());
        ui.label(RichText::new(format!("Slot {} · {}", i + 1, PLACES[i])).strong());
        // What the slot holds.
        ui.horizontal(|ui| {
            ui.add_sized([96., 18.], egui::Label::new("Holds"));
            let mut picked = None;
            let label = |k: Kind| match k {
                Kind::Command => "Command",
                Kind::SubPie => "Sub-pie",
                Kind::Empty => "Empty",
            };
            egui::ComboBox::from_id_salt(("pie-slot-kind", &id, i))
                .selected_text(label(kind))
                .show_ui(ui, |ui| {
                    for k in [Kind::Command, Kind::SubPie, Kind::Empty] {
                        if ui.selectable_label(k == kind, label(k)).clicked() {
                            picked = Some(k);
                        }
                    }
                });
            match picked {
                Some(Kind::Empty) if kind != Kind::Empty => {
                    self.execute(&format!("operandi.form.slot index:{i} form:{id}"));
                }
                // A command slot starts from the typed line below; a
                // sub-pie from the pick below.
                Some(k) if k != kind => {
                    self.form_editor.pie_kind = Some((id.clone(), i, k == Kind::SubPie))
                }
                _ => {}
            }
        });
        let wanted = match &self.form_editor.pie_kind {
            Some((f, at, sub)) if *f == id && *at == i => Some(*sub),
            _ => None,
        };
        let shows_sub = wanted.unwrap_or(kind == Kind::SubPie);
        let shows_line = wanted.map_or(kind == Kind::Command, |sub| !sub);
        let label = control
            .as_ref()
            .map(|c| c.label.clone())
            .unwrap_or_default();
        if shows_line {
            self.pie_line_field(ui, &id, i, control.as_ref(), &label);
        }
        if shows_sub {
            let current = match control.as_ref().and_then(|c| c.form.as_ref()) {
                Some(FormRef::Id(sub)) => sub.clone(),
                _ => String::new(),
            };
            // UIC-310: only pies that don't lead back to this one.
            let choices: Vec<String> = self
                .pie_ids()
                .into_iter()
                .filter(|p| {
                    *p != pie_menu::LAYOUTS && !crate::pie_forms::would_loop(&self.forms, &id, p)
                })
                .collect();
            ui.horizontal(|ui| {
                ui.add_sized([96., 18.], egui::Label::new("Sub-pie"));
                let mut picked = None;
                egui::ComboBox::from_id_salt(("pie-slot-sub", &id, i))
                    .selected_text(if current.is_empty() {
                        "Choose a pie…"
                    } else {
                        current.as_str()
                    })
                    .show_ui(ui, |ui| {
                        if choices.is_empty() {
                            ui.weak("No other pie can go here");
                        }
                        for p in &choices {
                            if ui.selectable_label(*p == current, p).clicked() {
                                picked = Some(p.clone());
                            }
                        }
                    });
                if let Some(p) = picked
                    && p != current
                {
                    self.form_editor.pie_kind = None;
                    self.execute(&format!(
                        "operandi.form.slot index:{i} sub:{p} label:{} form:{id}",
                        word(&label)
                    ));
                }
            });
        }
        if kind != Kind::Empty {
            self.form_text_field(ui, "Label", &id, Some(i), "label", &label);
            let icon = control
                .as_ref()
                .and_then(|c| c.icon.clone())
                .unwrap_or_default();
            self.form_text_field(ui, "Icon", &id, Some(i), "icon", &icon);
            if !icon.is_empty() && icons::Icon::from_slug(&icon).is_none() {
                ui.colored_label(theme::error(), format!("No icon called {icon}"));
            }
        }
    }

    /// A command slot's line: typed (set on Enter or leaving the field) or
    /// picked from the Command History (UIC-160). The label stays.
    fn pie_line_field(
        &mut self,
        ui: &mut Ui,
        id: &str,
        i: usize,
        control: Option<&Control>,
        label: &str,
    ) {
        let current = control.and_then(|c| c.command.clone()).unwrap_or_default();
        let key = (id.to_owned(), Some(i), "pie-line".to_owned());
        let mut chosen = None;
        ui.horizontal(|ui| {
            ui.add_sized([96., 18.], egui::Label::new("Command"));
            let mut text = self
                .form_editor
                .drafts
                .get(&key)
                .cloned()
                .unwrap_or_else(|| current.clone());
            let r = ui.add(
                egui::TextEdit::singleline(&mut text)
                    .hint_text("Command line")
                    .desired_width(ui.available_width() - 80.),
            );
            crate::numeric_tab::register(ui, &r, None);
            if r.changed() {
                self.form_editor.drafts.insert(key.clone(), text.clone());
            }
            if r.lost_focus() {
                self.form_editor.drafts.remove(&key);
                if text.trim() != current && !text.trim().is_empty() {
                    chosen = Some(text.trim().to_owned());
                }
            }
            let recent = self.recent_lines();
            ui.menu_button("History ▾", |ui| {
                if recent.is_empty() {
                    ui.weak("No commands run yet");
                }
                for line in recent {
                    if ui.button(&line).clicked() {
                        chosen = Some(line);
                        ui.close();
                    }
                }
            });
        });
        if let Some(line) = chosen {
            self.form_editor.pie_kind = None;
            // A slot that was a command keeps its label; a new one shows
            // its line until it gets one.
            let label = if current.is_empty() { "" } else { label };
            self.execute(&format!(
                "operandi.form.slot index:{i} line:{} label:{} form:{id}",
                word(&line),
                word(label)
            ));
        }
        if !current.is_empty() && self.line_state(&current).is_none() {
            ui.colored_label(
                theme::error(),
                "No command has this line: the slot shows red and does nothing (UIC-221)",
            );
        }
    }

    /// The Command History's last distinct lines that worked, newest
    /// first, leaving out the Form Editor's own.
    pub(crate) fn recent_lines(&self) -> Vec<String> {
        self.history
            .log
            .records
            .iter()
            .rev()
            .filter(|r| {
                let l = r.line.to_ascii_lowercase();
                r.ok && !l.starts_with("operandi.form.") && !l.starts_with("operandi.pie.")
            })
            .map(|r| r.line.clone())
            .fold(Vec::new(), |mut v, l| {
                if !v.contains(&l) && v.len() < 20 {
                    v.push(l);
                }
                v
            })
    }
}
