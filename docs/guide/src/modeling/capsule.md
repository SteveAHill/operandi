<!-- help-ids: Capsule, prim.capsule, Capsule Unit, Capsule New Unit ; template: tool ; since: 0.1.0 -->
# Capsule

**Capsule** adds a pill shape: a straight body with a rounded end at each side.

![A capsule 0.7 m across and 1.8 m long, with its Tool Properties](../images/capsule.png)

## How to use

1. Make sure the mesh you want to add to is selected in the **Items list**.
2. Click **Capsule** in the **Primitives** section of the **Create** tool tab.
3. Drag in the **3D view**. The drag draws the capsule's base flat on the
   **Work Plane**, centred on where you press.
4. Press on the dot in the middle of the flat base and drag up to give it its height.
5. Fine-tune it: drag its handles, or type values in **Tool Properties**.
6. Press **Q** (or **Esc**), or click **Capsule** again, to drop the tool
   and keep the capsule.

To draw it the same size in every direction instead, hold **Control** before
you start the drag: the press is the centre and the pointer sets the
size.

## Handles

Once the capsule has height, it shows its handles in the **3D view**:

- **Arrows** at the centre move it along X, Y or Z.
- **Squares** at the ends of each axis resize it along that axis. The capsule's far side stays where it is, so dragging a square stretches it toward that side.

A handle that points straight at you is hidden; orbit the view to reach it.

## Without a drag

- **Shift**-click **Capsule** adds a 1 m capsule as a new mesh item at
  the scene's origin.
- **Control**-click **Capsule** adds a 1 m capsule to the selected mesh, at
  its origin.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Position** | Where the centre is (X, Y, Z). | 0 m |
| **Radius** | Half the size along X, Y and Z; along the **Axis** it is half the length. | 0.25, 0.75, 0.25 m |
| **Sides** | How many polygons go round the axis. | 24 |
| **Segments** | How many rows of polygons the straight body has. | 1 |
| **End Segments** | How many rows each rounded end has. | 5 |
| **End Size** | How long each rounded end is. | 0.25 m |
| **Ends** | **Polar**: each end closes in a point. **Quad Grid**: each end is filled with four-sided polygons only. | Polar |
| **Bulge** | Swells the body. | 0 |
| **Axis** | The axis from end to end. | Y |
| **Make UVs** | Gives the shape a UV layout in the **Texture** UV map, for textures. | On |

- **Apply** adds the capsule to the mesh and keeps the tool, so you can
  make another.
- **Drop** adds it and closes the tool.
- **Cancel** throws the preview away.
- **Reset** puts every value back to its starting value.

## Tips

- Right-drag in the **3D view**, away from the handles: left and right change **Sides**, up and down change **Segments**.

## Shaping it with a falloff

A falloff can taper, bend or twist the capsule while you make it, as with the
Cube, Sphere and Cylinder. The falloff's weight decides how much each ring
of points changes: full where the weight is 1, not at all where it is 0.
More **Segments** give a smoother result.

1. With the capsule tool open, pick a **Linear**, **Radial**, **Cylinder** or
   **Noise** falloff from the **Falloff** menu. The tool stays open.
2. Open the **Falloff** section in **Tool Properties** and set what the
   weight does, for example **Scale** 0% for a point at the full end.
3. Drag the falloff's handles to move it. The capsule updates as you go, and
   as you change its own fields.
4. **Apply** or **Drop** to keep the shaped capsule. **Undo** takes it back
   in one step.

| Name | What it does | Default |
|---|---|---|
| **Scale** | The rings' size where the weight is full; 100% leaves them as they are. | 50% |
| **Offset** | How far the rings move where the weight is full (X, Y, Z). | 0 m |
| **Twist** | How far the rings turn about the axis where the weight is full. | 0° |
| **Sample At** | **Spine** reads one weight per ring on the axis, so rings stay round. **Points** reads each point's own weight, so a Noise or Radial falloff dents the rings. | Spine |
| **Spacing** | Always **Even**: moving the capsule's rings would change its shape, so **Follow Falloff** is greyed. | Even |

The rows stay grey until a falloff that can shape new geometry is on.
Falloffs that weigh points that already exist (**Selection**, **Element**,
**Vertex Map** and the painted ones) can't, and the rows say so.

## In the Mesh Stack

You can also add a Capsule as a step of a mesh's **Mesh Stack**: choose
**Capsule** under **Create > Primitives** in the stack's **Add Operator**
list. It has the same properties as the tool, and the tool and the stack
step build exactly the same capsule from the same values, so you can
change it later without rebuilding the mesh. A **Falloff** step below it in the stack shapes it (see Mesh Stack tool
steps): its **Falloff** rows work as above, with that step's falloff. Scripts
add it with `meshop.create type:prim.capsule.item`.

With the step selected in the stack and no tool on, its handles show in the
**3D view**. Drag an arrow to move it along that axis, a plane square or the centre dot to move it across a plane, and a size square at its edge to resize it, and its falloff's handles to change the falloff. Each drag is one undo step. A right-drag anywhere else changes its counts, as with the tool: left and right for **Sides**, up and down for **Segments**.

## Related

- [Primitives](../interface/window.md): the **Toolbox** on the **Create** tab
- [Interface](../interface/window.md): the **Work Plane** and **Tool Properties**
- [Sphere](sphere.md), [Cylinder](cylinder.md)
