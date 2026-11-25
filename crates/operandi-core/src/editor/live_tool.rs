//! The live tool session between activate and drop: primitives, transform
//! steps, retargeting, idle checks and unit insertion.
use super::*;

impl Editor {
    /// A live Cube, Sphere or Cylinder: like the transform tools it keeps
    /// running through Undo/Redo and restarts on the restored scene.
    pub(super) fn live_primitive_tool(&self) -> Option<Tool> {
        self.live
            .as_ref()
            .map(|l| l.tool)
            .filter(|t| t.is_primitive())
    }

    /// Throw away a primitive's preview that isn't a step yet, keeping the
    /// tool active. Numeric sessions keep their values; a drawn Cube, Sphere
    /// or Cylinder restarts empty, as after activation. Returns the tool's
    /// name, or None when no such preview shows.
    pub(super) fn discard_primitive_preview(
        &mut self,
        numeric_only: bool,
    ) -> Result<Option<String>> {
        let Some(label) = self.primitive_session().and_then(|s| s.pending_preview) else {
            return Ok(None);
        };
        if numeric_only && self.live.is_some() {
            return Ok(None);
        }
        if let Some(s) = &mut self.ellipsoid {
            s.preview = None;
        } else if let Some(s) = &mut self.shape {
            s.preview = None;
        } else if let Some(s) = &mut self.cone {
            s.preview = None;
        } else if let Some(l) = self.live.take() {
            self.scene = l.base;
            if let Some(ids) = l.identity_base {
                self.identities.current = ids;
                self.identities.observe(&self.scene)?;
            }
            self.resume_transform(l.tool);
        }
        Ok(Some(label.to_string()))
    }

    /// After Undo/Redo changed the scene under a numeric primitive session:
    /// keep it on its mesh when that mesh can still take the shape, close
    /// it otherwise (MTU-Q6, provisional). Its values stay; nothing is
    /// previewed until a value changes.
    pub(super) fn rebase_numeric_primitive(&mut self) {
        if let Some(s) = &self.ellipsoid
            && self.ellipsoid_target_ok(s.target).is_err()
        {
            self.ellipsoid = None;
        }
        if let Some(s) = &self.shape
            && self.shape_target_ok(s.target, s.kind()).is_err()
        {
            self.shape = None;
        }
        if let Some(s) = &self.cone
            && !self
                .scene
                .items
                .iter()
                .any(|i| i.id == s.target && self.scene.shows(i))
        {
            self.cone = None;
        }
        for preview in [
            self.ellipsoid.as_mut().map(|s| &mut s.preview),
            self.shape.as_mut().map(|s| &mut s.preview),
            self.cone.as_mut().map(|s| &mut s.preview),
        ]
        .into_iter()
        .flatten()
        {
            *preview = None;
        }
    }

    /// A live transform tool shows a change not yet recorded as a step
    /// (a typed field value, or a drag still in progress).
    pub fn transform_change_pending(&self) -> bool {
        self.live
            .as_ref()
            .is_some_and(|l| l.tool.is_transform() && l.base != self.scene)
    }

    pub(super) fn live_transform_tool(&self) -> Option<Tool> {
        self.live
            .as_ref()
            .map(|l| l.tool)
            .filter(|t| t.is_transform())
    }

    /// Record the live transform tool's change since its last step as one
    /// undo step, then keep the tool live from the result with zero offset.
    /// The center follows a move so the handle stays on the moved
    /// selection. Provisional (Jason, 2026-11-03; HND-Q03): the tool's
    /// fields restart from zero after each drag.
    pub(crate) fn commit_transform_step(&mut self) -> Result<()> {
        let live = self
            .live
            .as_ref()
            .filter(|l| l.tool.is_transform())
            .ok_or("No active transform tool")?;
        if let Some(error) = &live.preview_error {
            return Err(format!("Correct the current transform first: {error}"));
        }
        if live.base == self.scene {
            return Ok(());
        }
        self.scene.validate()?;
        self.last_transform_step = Some(Box::new(crate::transform_repeat::TransformStep {
            tool: live.tool,
            base: live.base.clone(),
            params: live.params.clone(),
            action_frame: live.action_frame,
            identity_base: live.identity_base.clone(),
            applied: live.applied,
            after: self.scene.clone(),
        }));
        let mut next = live.clone();
        if matches!(live.tool, Tool::Move | Tool::Transform)
            && let Some(frame) = next.action_frame.as_mut()
        {
            frame.center += frame.basis * live.params.delta;
        }
        next.params.delta = if live.tool == Tool::Scale {
            DVec3::ONE
        } else {
            DVec3::ZERO
        };
        next.params.rotation_delta = DVec3::ZERO;
        next.params.scale_delta = DVec3::ONE;
        // Box Transform (UVT-43): the next drag starts from the moved box;
        // a set pivot moves with it.
        let b = live.params.uv_box;
        next.params.uv_box = crate::uv_box::BoxTransform {
            pivot: b.pivot.map(|p| p + b.offset),
            show_rotation_handle: b.show_rotation_handle,
            snap_angle: b.snap_angle,
            ..Default::default()
        };
        next.params.tweak = None;
        next.params.tweak_plan = None;
        // TK-30: the drag's edit keys when it ends, not only on Drop.
        let base = std::mem::take(&mut next.base);
        self.auto_key_from(&base);
        next.base = self.scene.clone();
        next.applied = false;
        // The drag stays a tweak of the session (Jason, 2026-11-04; this
        // replaces TWK-24's step per Tweak drag); Drop records the session
        // as one step.
        next.identity_base = Some(self.identities.current.clone());
        next.identity_preview = None;
        self.live = Some(next);
        Ok(())
    }

    /// After Undo/Redo with a transform tool live: start it again on the
    /// restored scene (its center comes from the restored selection). If
    /// the restored scene can't host the tool it stays dropped.
    pub(super) fn resume_transform(&mut self, tool: Tool) {
        if self.live.is_none() {
            let _ = self.handle_activate(tool);
        }
    }

    /// Change the action center and axis.
    pub(super) fn handle_action_context(&mut self, context: crate::ActionContext) -> Result<()> {
        if self.cone.is_some() {
            return Err("Numeric Cone uses world coordinates; context changes during this session remain uncharacterized".into());
        }
        // Resolve before publishing context so invalid Selection requests
        // preserve the live stage, previous context and history atomically.
        let frame = self.action_context_frame(context)?;
        let previous_context = self.action_context;
        let previous_frame = self.live.as_ref().and_then(|live| live.action_frame);
        let previous_bend_center = self.live.as_ref().map(|live| live.params.bend.base);
        let parameters = self
            .live
            .as_ref()
            .filter(|live| {
                live.applied
                    && matches!(
                        live.tool,
                        Tool::Move
                            | Tool::Rotate
                            | Tool::Scale
                            | Tool::Transform
                            | Tool::EdgeExtend
                    )
            })
            .map(|live| live.params.clone());
        self.action_context = context;
        if let Some(live) = &mut self.live {
            live.action_frame = frame;
            live.placed_centre = None;
        }
        // Bend: its centre is the action centre, so a new centre mode moves
        // it (and re-bends from there once applied).
        let parameters = parameters.or_else(|| {
            let live = self.live.as_mut().filter(|l| l.tool == Tool::Bend)?;
            let center = live.action_frame?.center;
            live.params.bend.base = center;
            live.applied.then(|| live.params.clone())
        });
        if let Some(parameters) = parameters
            && let Err(error) = self.preview(parameters, false)
        {
            // Transform preview validates its local candidate before
            // publishing scene/params. Restore only the frame/context
            // staged above; never clone or commit the undo history.
            self.action_context = previous_context;
            if let Some(live) = &mut self.live {
                live.action_frame = previous_frame;
                if let Some(center) = previous_bend_center {
                    live.params.bend.base = center;
                }
            }
            return Err(error);
        }
        Ok(())
    }

    /// Select the polygons of a named polygon set.
    pub(super) fn handle_select_polygon_set(&mut self, name: String) -> Result<()> {
        self.require_idle()?;
        let item = self.scene.active().ok_or("Select a mesh")?;
        if !self.scene.shows(item) {
            return Err("Show the mesh before recalling components".into());
        }
        let members = item
            .mesh
            .attributes
            .polygon_sets
            .get(&name)
            .ok_or("Named polygon set not found")?;
        let ids = self
            .identities
            .current
            .meshes
            .get(&item.id)
            .ok_or("Missing identity view")?;
        let faces = ids
            .logical_faces
            .iter()
            .filter_map(|id| ids.faces.iter().position(|x| x == id))
            .filter(|i| members.contains(&(*i as u32)))
            .map(|i| i as u32)
            .collect();
        let before = self.scene.clone();
        self.scene.selection.mode = Mode::Polygon;
        self.scene.selection.faces = faces;
        self.scene.selection.vertices.clear();
        self.scene.selection.edges.clear();
        if before != self.scene {
            self.record_selection("Selection".into(), before);
        }
        Ok(())
    }

    /// A camera, light or Planar locator channel edited from the live
    /// item tool (CL-14, planar_texture.rs). Kept out of `dispatch` so its
    /// scene copies don't grow that recursive function's frame.
    #[inline(never)]
    pub(super) fn tool_item_channel(
        &mut self,
        item: u64,
        edit: &crate::camera_light::ItemChannelEdit,
    ) -> Result<()> {
        use crate::camera_light::ItemChannelEdit;
        let refusal = || -> String {
            if matches!(edit, ItemChannelEdit::Transform(_)) {
                "Use Move, Rotate, Scale or Transform on the Texture Locator".into()
            } else {
                "Use Move, Rotate or Transform on the camera or light".into()
            }
        };
        let live = self.live.as_ref().ok_or_else(refusal)?;
        let tool_ok = match edit {
            ItemChannelEdit::Transform(_) => live.tool.transforms_items(),
            _ => crate::camera_light::tool_shows_item_channels(live.tool),
        };
        if !tool_ok
            || self.scene.selection.mode != Mode::Item
            || self.scene.selection.item != Some(item)
        {
            return Err(refusal());
        }
        // Both the scene and the tool's base, so the tool's next
        // preview (built from its base) keeps the change.
        let mut scene = Box::new(self.scene.clone());
        scene.apply_item_channel(item, edit)?;
        let mut base = Box::new(live.base.clone());
        match edit {
            ItemChannelEdit::Transform(t) => {
                // PD-Q06: one item only; several selected move about a
                // shared pivot, which this doesn't invert.
                if live.base.item_transform_targets().is_some() {
                    return Err("Select only the Texture Locator to drag its projection".into());
                }
                let b = crate::planar_texture::base_for_tool(t, live.tool, &live.params)?;
                base.apply_item_channel(item, &ItemChannelEdit::Transform(b))?;
            }
            _ => base.apply_item_channel(item, edit)?,
        }
        scene.validate()?;
        self.scene = *scene;
        if let Some(live) = self.live.as_mut() {
            live.base = *base;
        }
        Ok(())
    }

    /// Re-target a live transform tool to a new item selection (see the
    /// route in `dispatch`). Kept out of `dispatch` so the staged copy
    /// doesn't grow every frame of that recursive function.
    #[inline(never)]
    pub(super) fn retarget_transform(&mut self, tool: Tool, cmd: Command) -> Result<()> {
        let live = self.live.as_ref().ok_or("No live tool")?;
        let negative_scale = live.params.negative_scale;
        let tweak_options = live.params.tweak_options;
        let mut staged = Box::new(self.clone());
        staged.dispatch(Command::Drop)?;
        if staged.has_active_task() || staged.i2_active() {
            return Err("Finish the current tool before changing the item selection".into());
        }
        staged.dispatch(cmd)?;
        if staged.dispatch(Command::Activate(tool)).is_ok()
            && let Some(live) = staged.live.as_mut()
        {
            live.params.negative_scale = negative_scale;
            live.params.tweak_options = tweak_options;
        }
        *self = *staged;
        Ok(())
    }

    /// Select a scene item (or none).
    pub(super) fn handle_select_item(&mut self, id: Option<u64>) -> Result<()> {
        self.require_idle()?;
        if id.is_some_and(|id| !self.scene.item(id).is_some()) {
            return Err("Missing mesh item".into());
        }
        self.scene.select_item(id);
        Ok(())
    }

    /// Sketch Extrude along an explicit world path in one step.
    pub(super) fn handle_sketch_extrude_once(&mut self, path: Vec<DVec3>) -> Result<()> {
        if self.live.is_some() {
            return Err("Finish or Cancel the active tool first".into());
        }
        if path.len() < 2 {
            return Err("Sketch Extrude requires a nonzero path".into());
        }
        let mut staged = self.clone();
        staged.dispatch(Command::Activate(Tool::SketchExtrude))?;
        let mut params = staged.live.as_ref().unwrap().params.clone();
        params.sketch_path = path;
        staged.dispatch(Command::Preview(params))?;
        staged.dispatch(Command::Drop)?;
        *self = staged;
        Ok(())
    }

    /// Change the live Extrude stage's numeric parameters.
    pub(super) fn handle_configure_extrude(&mut self, p: Parameters) -> Result<()> {
        let live = self.live.as_ref().ok_or("Arm Extrude first")?;
        if live.tool != Tool::Extrude {
            return Err("Arm Extrude first".into());
        }
        if p.extrude_uvs != live.params.extrude_uvs
            && (!p.extrude_uv_selected
                || !p.extrude_uv_map.as_ref().is_some_and(|name| {
                    self.scene
                        .active()
                        .is_some_and(|i| i.mesh.uv_maps.contains_key(name))
                }))
        {
            return Err("Select a UV map before changing Make UVs".into());
        }
        if !p.extent.is_finite() || p.extrude_sides == 0 || p.extrude_sides > 1_000_000 {
            return Err("Invalid Extrude numeric configuration".into());
        }
        if live.applied {
            self.preview(p.clone(), false)?;
        } else {
            self.live.as_mut().unwrap().params = p.clone();
        }
        *self.presets.get_mut::<crate::ExtrudePreset>() = p;
        Ok(())
    }

    /// Stage the drop in a clone. The live task, history and revision remain
    /// owned by this editor until the atomic filesystem write succeeds.
    pub fn save_committed(&mut self, path: &std::path::Path) -> Result<()> {
        if self.live.as_ref().is_some_and(|l| {
            l.params
                .radial
                .is_some_and(|p| p.radius.min_element() == 0.)
        }) {
            return Err("Flat Sphere/Cylinder draft has no committed topology; enter positive Radius values or Cancel before Save".into());
        }
        let mut staged = self.clone();
        staged.dispatch(Command::Drop)?;
        if let Err(e) = crate::persistence::save(path, &staged.scene) {
            return Err(format!("Save {}: {e}", path.display()));
        }
        *self = staged;
        Ok(())
    }
    pub fn require_idle(&self) -> Result<()> {
        if self.bridge.is_some() {
            return Err("Drop or Cancel Bridge first".into());
        }
        if self.reduction.is_some() {
            return Err("Drop or Cancel Polygon Reduction first".into());
        }
        if self.live.is_some()
            || self.cone.is_some()
            || self.ellipsoid.is_some()
            || self.shape.is_some()
            || self.spike.is_some()
        {
            Err("Drop or Cancel the active tool first".into())
        } else {
            Ok(())
        }
    }
    /// PM-UP01..06: stage geometry, destination and validation before publishing
    /// anything. Do not activate/drop a tool or overwrite remembered dimensions.
    pub(super) fn insert_unit(&mut self, tool: Tool, destination: UnitDestination) -> Result<()> {
        self.require_idle()?;
        if self.selection_start.is_some() {
            return Err("Finish the selection gesture before unit insertion".into());
        }
        if !tool.is_primitive() {
            return Err("Unit insertion requires Cube, Sphere or Cylinder".into());
        }
        self.scene.validate()?;
        let mut scene = self.scene.clone();
        if destination == UnitDestination::NewMesh {
            let next = scene.next_id.max(self.next_item_id);
            if next >= u64::MAX - 1 || scene.items.len() >= crate::MAX_SCENE_ITEMS {
                return Err("Scene item/identity budget exhausted".into());
            }
            scene.next_id = next;
            scene.new_mesh()?;
            // The new item is named after the shape (unit-primitive rule 10).
            scene.active_mut().ok_or("Missing new mesh")?.name = tool.label().into();
        }
        let item = scene
            .active()
            .ok_or("Select an eligible mesh for unit insertion; Shift-click creates a new mesh")?;
        if item.backdrop.is_some() {
            return Err(BACKDROP_NO_GEOMETRY.into());
        }
        if !item.is_mesh() {
            return Err(LOCATOR_NO_GEOMETRY.into());
        }
        if !scene.shows(item) {
            return Err("Unit insertion requires one visible mesh".into());
        }
        let item_id = item.id;
        let make_uvs = !self.primitive_uvs_off[primitive_slot(tool)];
        // Unit primitives ignore the tools' last-used settings and use their
        // reset shapes (unit-primitive-2026-11-05 rules 9 and 11).
        let geometry = if tool == Tool::Box {
            crate::CubeShape {
                make_uvs,
                ..Default::default()
            }
            .mesh()?
        } else {
            let mut radial = tool
                .initial_radial()
                .ok_or("Missing unit primitive profile")?;
            // The unit Cylinder is 1.5 m tall like the tool's reset size.
            radial.radius = if tool == Tool::Cylinder {
                DVec3::new(0.5, 0.75, 0.5)
            } else {
                DVec3::splat(0.5)
            };
            radial_mesh(
                tool,
                DVec3::ZERO,
                radial,
                &Parameters {
                    make_uvs,
                    ..Default::default()
                },
                None,
            )?
        };
        geometry.validate()?;
        let geometry =
            crate::user_defaults::primitive_symmetry(self.active_symmetry()).doubled(geometry)?;
        // Made at the world origin like the drawn tools, then mapped into a
        // moved mesh's local space (provisional, PRIM-MOVED-Q01).
        let geometry = scene.mesh_world_to_local(item_id, geometry)?;
        crate::append_geometry(
            &mut scene.active_mut().ok_or("Select a mesh")?.mesh,
            geometry,
        )?;
        scene.validate()?;
        let before = std::mem::replace(&mut self.scene, scene);
        if destination == UnitDestination::NewMesh {
            self.next_item_id = self.scene.next_id;
        }
        self.record(format!("Unit {}", tool.label()), before);
        Ok(())
    }
}
