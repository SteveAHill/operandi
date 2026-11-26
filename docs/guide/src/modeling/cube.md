<!-- help-ids: Cube, prim.cube, Cube Unit, Cube New Unit ; template: tool ; since: 0.1.0 -->
# Cube

**Cube** adds a box: six sides, each split into as many rows of polygons as you ask for.

![A cube 1.5 m on every side, with its handles and its Tool Properties](../images/cube.png)

## How to use

1. Make sure the mesh you want to add to is selected in the **Items list**.
2. Click **Cube** in the **Primitives** section of the **Create** tool tab.
3. Drag in the **3D view**. The drag draws the cube's base flat on the
   **Work Plane**, from where you press to where you let go.
4. Drag the handle on top of the flat base up or down to give it its height.
5. Fine-tune it: drag its handles, or type values in **Tool Properties**.
6. Press **Q** (or **Esc**), or click **Cube** again, to drop the tool
   and keep the cube.

To draw it the same size in every direction instead, hold **Control** before
you start the drag: the press is the centre and the pointer sets the
size.

## Without a drag

- **Shift**-click **Cube** adds a 1 m cube as a new mesh item at
  the scene's origin.
- **Control**-click **Cube** adds a 1 m cube to the selected mesh, at
  its origin.

## Handles

| Handle | Drag it to |
|---|---|
| Square on a side | Move that side; the box grows or shrinks (see **Handle Mode**) |
| Arrows in the middle | Move the whole box |

## Properties

| Name | What it does | Default |
|---|---|---|
| **Position** | Where the box's centre is (X, Y, Z). | 0 m |
| **Size** | How big the box is along X, Y and Z. A size of 0 makes one flat polygon. | 1 m |
| **Segments** | How many rows of polygons each side has, along X, Y and Z. | 1 |
| **Make UVs** | Gives the shape a UV layout in the **Texture** UV map, for textures. | On |
| **Handle Mode** | What a side handle does: **Resize** moves only that side; **Resize from Center** moves both opposite sides; **Move All** moves the box. | Resize |
| **Min**, **Max** | The box's lowest and highest corner. Type here to place a side exactly. | from Position and Size |

Rows in grey aren't available yet; they show their value only. They are **Radius**, **Radius Segments**, **Sharp**, **Bulge** and **Axis** (rounded corners), **Distortion Compensation**, **Gaps**, **Show Corner**, and **Apply**.

- **Reset Tool Attributes** puts **Sides** and **Segments** back to their
  starting values; later uses start from them too.
- **Drop Tool** keeps the cube and closes the tool. **Cancel** takes
  back the last change.

## Tips

- A negative **Size** gives the same box as a positive one.
- Change **Segments** before you add bevels or loops, so the sides already
  have the rows you need.

## Shaping it with a falloff

A falloff can taper, bend or twist the cube while you make it. The
falloff's weight decides how much each ring of points changes: full where
the weight is 1, not at all where it is 0. More **Segments** give a
smoother result.

1. With the cube still being drawn, pick a **Linear**, **Radial**,
   **Cylinder** or **Noise** falloff from the **Falloff** menu. The tool
   stays open.
2. Open the **Falloff** section in **Tool Properties** and set what the
   weight does, for example **Scale** 0% for a point at the full end.
3. Drag the falloff's handles to move it, or click **Fit Falloff to Shape**
   to run a Linear falloff from the cube's base (none) to its top (full).
4. Go back to the cube's own handles and fields whenever you like. The
   shape updates as you change either.
5. Drop the tool to keep the shaped cube. **Undo** takes it back in one
   step.

| Name | What it does | Default |
|---|---|---|
| **Scale** | The rings' size where the weight is full; 100% leaves them as they are. | 50% |
| **Offset** | How far the rings move where the weight is full (X, Y, Z). | 0 m |
| **Twist** | How far the rings turn about the axis where the weight is full. | 0° |
| **Sample At** | **Spine** reads one weight per ring on the axis, so rings stay round. **Points** reads each point's own weight, so a Noise or Radial falloff dents the rings. | Spine |
| **Spacing** | **Even** keeps the rings where they are. **Follow Falloff** packs them where the falloff changes fastest, so a curved profile stays smooth with fewer **Segments**. | Even |
| **Axis** | The axis the falloff shapes the cube along. | Y |

The rows stay grey until a falloff that can shape new geometry is on.
Falloffs that weigh points that already exist (**Selection**, **Element**,
**Vertex Map** and the painted ones) can't, and the rows say so.

## In the Mesh Stack

You can also add a Cube as a step of a mesh's **Mesh Stack**: choose
**Cube** under **Create > Primitives** in the stack's **Add Operator**
list. It has the same properties as the tool, and the tool and the stack
step build exactly the same cube from the same values, so you can
change it later without rebuilding the mesh. A **Falloff** step below it in the stack shapes it (see Mesh Stack tool
steps): its **Falloff** rows work as above, with that step's falloff. Rows that only steer the
tool's handles (**Handle Mode**, **Min** and **Max**) aren't there. Scripts
add it with `meshop.create type:prim.cube.item`.

With the step selected in the stack and no tool on, its handles show in the
**3D view**. Drag an arrow to move it along that axis, a plane square or the centre dot to move it across a plane, and a size square at its edge to resize it, and its falloff's handles to change the falloff (they work as the tool falloff's do: points, arrows and Size boxes). Each drag is one undo step. A right-drag anywhere else changes its segments, as with the tool: left and right for **Segments X**, up and down for **Segments Z**.

## Related

- [Primitives](../interface/window.md): the **Toolbox** on the **Create** tab
- [Interface](../interface/window.md): the **Work Plane** and **Tool Properties**
- [Sphere](sphere.md), [Cylinder](cylinder.md)
