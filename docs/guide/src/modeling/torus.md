<!-- help-ids: Torus, prim.torus, prim.toroid, Torus Unit, Torus New Unit ; template: tool ; since: 0.1.0 -->
# Torus

**Torus** adds a ring, like a doughnut: a round cross-section swept round an axis.

![A torus with Ring Radius 0.75 m and Cross Section 0.25 m, with its Tool Properties](../images/torus.png)

## How to use

1. Make sure the mesh you want to add to is selected in the **Items list**.
2. Click **Torus** in the **Primitives** section of the **Create** tool tab.
3. Drag in the **3D view**. The drag draws the torus's base flat on the
   **Work Plane**, centred on where you press.
4. Press on the dot in the middle of the flat base and drag up to give it its height.
5. Fine-tune it: drag its handles, or type values in **Tool Properties**.
6. Press **Q** (or **Esc**), or click **Torus** again, to drop the tool
   and keep the torus.

To draw it the same size in every direction instead, hold **Control** before
you start the drag: the press is the centre and the pointer sets the
size.

## Without a drag

- **Shift**-click **Torus** adds a 1 m torus as a new mesh item at
  the scene's origin.
- **Control**-click **Torus** adds a 1 m torus to the selected mesh, at
  its origin.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Position** | Where the centre is (X, Y, Z). | 0 m |
| **Radius** | The ring's overall half-size along X, Y and Z. | 0.25 m |
| **Sides** | How many steps go round the ring. | 24 |
| **Segments** | How many steps go round the cross-section. | 12 |
| **Ring Radius** | Distance from the centre to the middle of the cross-section. | 0.5 m |
| **Cross Section** | Half the cross-section's width. | 0.25 m |
| **Height** | Half the cross-section's height. | 0.25 m |
| **Hole Size** | **Ring Radius** divided by **Cross Section**. Change it to open or close the hole; **Ring Radius** follows. | 2 |
| **Bulge Top**, **Side** | Push the cross-section toward a square. | 0 |
| **Axis** | The axis the ring goes round. | Y |
| **Make UVs** | Gives the shape a UV layout in the **Texture** UV map, for textures. | On |

- **Apply** adds the torus to the mesh and keeps the tool, so you can
  make another.
- **Drop** adds it and closes the tool.
- **Cancel** throws the preview away.
- **Reset** puts every value back to its starting value.

## Tips

- Right-drag in the **3D view**, away from the handles: left and right change **Sides**, up and down change **Segments**.
- **Radius**, **Ring Radius**, **Cross Section**, **Height** and **Hole
  Size** are linked: change one and the others that depend on it follow.
  A **Hole Size** of 1 closes the hole to the axis.

## Shaping it with a falloff

A falloff can taper, bend or twist the torus while you make it, as with the
Cube, Sphere and Cylinder. The falloff's weight decides how much each ring
of points changes: full where the weight is 1, not at all where it is 0.
More **Segments** give a smoother result.

1. With the torus tool open, pick a **Linear**, **Radial**, **Cylinder** or
   **Noise** falloff from the **Falloff** menu. The tool stays open.
2. Open the **Falloff** section in **Tool Properties** and set what the
   weight does, for example **Scale** 0% for a point at the full end.
3. Drag the falloff's handles to move it. The torus updates as you go, and
   as you change its own fields.
4. **Apply** or **Drop** to keep the shaped torus. **Undo** takes it back
   in one step.

| Name | What it does | Default |
|---|---|---|
| **Scale** | The rings' size where the weight is full; 100% leaves them as they are. | 50% |
| **Offset** | How far the rings move where the weight is full (X, Y, Z). | 0 m |
| **Twist** | How far the rings turn about the axis where the weight is full. | 0° |
| **Sample At** | **Spine** reads one weight per ring on the axis, so rings stay round. **Points** reads each point's own weight, so a Noise or Radial falloff dents the rings. | Spine |
| **Spacing** | Always **Even**: moving the torus's rings would change its shape, so **Follow Falloff** is greyed. | Even |

The rows stay grey until a falloff that can shape new geometry is on.
Falloffs that weigh points that already exist (**Selection**, **Element**,
**Vertex Map** and the painted ones) can't, and the rows say so.

## In the Mesh Stack

You can also add a Torus as a step of a mesh's **Mesh Stack**: choose
**Torus** under **Create > Primitives** in the stack's **Add Operator**
list. It has the same properties as the tool, and the tool and the stack
step build exactly the same torus from the same values, so you can
change it later without rebuilding the mesh. A **Falloff** step below it in the stack shapes it (see Mesh Stack tool
steps): its **Falloff** rows work as above, with that step's falloff. **Hole Size** isn't there:
it follows from **Ring Radius** and **Cross Section**, which are. Scripts
add it with `meshop.create type:prim.toroid.item`.

With the step selected in the stack and no tool on, its handles show in the
**3D view**. Drag an arrow to move it along that axis, a plane square or the centre dot to move it across a plane, and a size square at its edge to resize it, and its falloff's handles to change the falloff. Each drag is one undo step. A right-drag anywhere else changes its counts, as with the tool: left and right for **Sides**, up and down for **Segments**.

## Related

- [Primitives](../interface/window.md): the **Toolbox** on the **Create** tab
- [Interface](../interface/window.md): the **Work Plane** and **Tool Properties**
- [Capsule](capsule.md)
