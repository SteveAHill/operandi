<!-- help-ids: Cylinder, prim.cylinder, Cylinder Unit, Cylinder New Unit ; template: tool ; since: 0.1.0 -->
# Cylinder

**Cylinder** adds a tube with closed ends: rings of polygons round an axis.

![A cylinder 1.2 m across and 1.5 m tall, with its handles and its Tool Properties](../images/cylinder.png)

## How to use

1. Make sure the mesh you want to add to is selected in the **Items list**.
2. Click **Cylinder** in the **Primitives** section of the **Create** tool tab.
3. Drag in the **3D view**. The drag draws the cylinder's base flat on the
   **Work Plane**, from where you press to where you let go.
4. Drag a side handle to give the flat draft its height, or type its **Radius**.
5. Fine-tune it: drag its handles, or type values in **Tool Properties**.
6. Press **Q** (or **Esc**), or click **Cylinder** again, to drop the tool
   and keep the cylinder.

To draw it the same size in every direction instead, hold **Control** before
you start the drag: the press is the centre and the pointer sets the
size.

## Without a drag

- **Shift**-click **Cylinder** adds a 1 m cylinder as a new mesh item at
  the scene's origin.
- **Control**-click **Cylinder** adds a 1 m cylinder to the selected mesh, at
  its origin.

## Handles

| Handle | Drag it to |
|---|---|
| Square on a side | Change the **Radius** along that axis; hold **Control** to move the cylinder instead |
| Arrows in the middle | Move the cylinder |

## Properties

| Name | What it does | Default |
|---|---|---|
| **Position** | Where the centre is (X, Y, Z). | 0 m |
| **Radius** | Half the size along X, Y and Z; along the **Axis** it is half the height. | 0.5, 0.75, 0.5 m |
| **Sides** | How many polygons go round the axis. | 24 |
| **Segments** | How many rows of polygons go from end to end. | 1 |
| **Polygon Type** | The kind of polygons made. | Face |
| **Top Cap**, **Bottom Cap** | How each end is closed: **None**, **Single Polygon**, **Quad Grid** or **Radial**. **Quad Grid** fills the end with a grid of quads, like the cone's base, so it subdivides cleanly; an odd number of **Sides** leaves one triangle. | Single Polygon |
| **Axis** | The axis from end to end. | Y |
| **Make UVs** | Gives the shape a UV layout in the **Texture** UV map, for textures. | On |
| **Handle Mode** | Side handles resize that side. | Resize |
| **Min**, **Max** | The lowest and highest corner. | from Position and Radius |

Rows in grey aren't available yet; they show their value only. They are **Edge Weight**, **Distortion Compensation**, **Gaps** and **Show Corner**.

- **Reset Tool Attributes** puts **Sides** and **Segments** back to their
  starting values; later uses start from them too.
- **Drop Tool** keeps the cylinder and closes the tool. **Cancel** takes
  back the last change.

## Tips

- **Top Cap** set to **None** leaves that end open, for a pipe or a cup.
- Right-drag in the **3D view**, away from the handles: left and right change **Sides**, up and down change **Segments**.

## Shaping it with a falloff

A falloff can taper, bend or twist the cylinder while you make it. The
falloff's weight decides how much each ring of points changes: full where
the weight is 1, not at all where it is 0. More **Segments** give a
smoother result.

1. With the cylinder still being drawn, pick a **Linear**, **Radial**,
   **Cylinder** or **Noise** falloff from the **Falloff** menu. The tool
   stays open.
2. Open the **Falloff** section in **Tool Properties** and set what the
   weight does, for example **Scale** 0% for a point at the full end.
3. Drag the falloff's handles to move it, or click **Fit Falloff to Shape**
   to run a Linear falloff from the cylinder's base (none) to its top (full).
4. Go back to the cylinder's own handles and fields whenever you like. The
   shape updates as you change either.
5. Drop the tool to keep the shaped cylinder. **Undo** takes it back in one
   step.

| Name | What it does | Default |
|---|---|---|
| **Scale** | The rings' size where the weight is full; 100% leaves them as they are. | 50% |
| **Offset** | How far the rings move where the weight is full (X, Y, Z). | 0 m |
| **Twist** | How far the rings turn about the axis where the weight is full. | 0° |
| **Sample At** | **Spine** reads one weight per ring on the axis, so rings stay round. **Points** reads each point's own weight, so a Noise or Radial falloff dents the rings. | Spine |
| **Spacing** | **Even** keeps the rings where they are. **Follow Falloff** packs them where the falloff changes fastest, so a curved profile stays smooth with fewer **Segments**. | Even |

The rows stay grey until a falloff that can shape new geometry is on.
Falloffs that weigh points that already exist (**Selection**, **Element**,
**Vertex Map** and the painted ones) can't, and the rows say so.

## In the Mesh Stack

You can also add a Cylinder as a step of a mesh's **Mesh Stack**: choose
**Cylinder** under **Create > Primitives** in the stack's **Add Operator**
list. It has the same properties as the tool, and the tool and the stack
step build exactly the same cylinder from the same values, so you can
change it later without rebuilding the mesh. A **Falloff** step below it in the stack shapes it (see Mesh Stack tool
steps): its **Falloff** rows work as above, with that step's falloff. Rows that only steer the
tool's handles (**Handle Mode**, **Min** and **Max**) aren't there. Scripts
add it with `meshop.create type:prim.cylinder.item`.

With the step selected in the stack and no tool on, its handles show in the
**3D view**. Drag an arrow to move it along that axis, a plane square or the centre dot to move it across a plane, and a size square at its edge to resize it, and its falloff's handles to change the falloff (they work as the tool falloff's do: points, arrows and Size boxes). Each drag is one undo step. A right-drag anywhere else changes its counts, as with the tool: left and right for **Sides**, up and down for **Segments**.

## Related

- [Primitives](../interface/window.md): the **Toolbox** on the **Create** tab
- [Interface](../interface/window.md): the **Work Plane** and **Tool Properties**
- [Cone](cone.md), [Tube](tube.md)
