<!-- help-ids: Cone, prim.cone, Cone Unit, Cone New Unit ; template: tool ; since: 0.1.0 -->
# Cone

**Cone** adds a cone: a round base that narrows to a point.

![A cone 1.2 m across and 1.8 m tall, with its Tool Properties](../images/cone.png)

## How to use

1. Make sure the mesh you want to add to is selected in the **Items list**.
2. Click **Cone** in the **Primitives** section of the **Create** tool tab.
3. Drag in the **3D view**. The drag draws the cone's base flat on the
   **Work Plane**, centred on where you press.
4. Press on the dot in the middle of the flat base and drag up to give it its height.
5. Fine-tune it: drag its handles, or type values in **Tool Properties**.
6. Press **Q** (or **Esc**), or click **Cone** again, to drop the tool
   and keep the cone.

To draw it the same size in every direction instead, hold **Control** before
you start the drag: the press is the centre and the pointer sets the
size.

## Without a drag

- **Shift**-click **Cone** adds a 1 m cone as a new mesh item at
  the scene's origin.
- **Control**-click **Cone** adds a 1 m cone to the selected mesh, at
  its origin.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Position** | Where the centre is (X, Y, Z). | 0 m |
| **Size** | Half the size along X, Y and Z; along the **Axis** it is half the height. | 0.5, 0.75, 0.5 m |
| **Sides** | How many polygons go round the axis. | 24 |
| **Segments** | How many rows of polygons go from base to tip. | 1 |
| **Polygon Type** | **Face**, **Subdivs** or **Catmull Clark** polygons. | Face |
| **Top Cap** | How the tip is built. The tip is a single point, so **None** and **Single Polygon** both make a fan of triangles there; **Quad Grid** joins them in pairs into quads (it needs an even number of **Sides**). **Radial** is greyed. | Single Polygon |
| **Bottom Cap** | How the base is closed: **None**, **Single Polygon**, **Quad Grid** or **Radial**. | Single Polygon |
| **Edge Weight %** | How sharp the rim stays when the polygons are subdivided. Greyed with **Face** polygons. | 100 |
| **Axis** | The axis from base to tip. | Y |
| **Make UVs** | Gives the shape a UV layout in the **Texture** UV map, for textures. | On |
| **Distortion Compensation** | Keeps UV tiles in proportion to the polygons' sizes. | Off |
| **Gaps %** | Space left round the UV layout. | 0 |
| **UV Map** | Which UV map gets the layout. | Texture |
| **Min**, **Max** | The lowest and highest corner. | from Position and Size |

- **Apply** adds the cone to the mesh and keeps the tool, so you can
  make another.
- **Drop** adds it and closes the tool.
- **Cancel** throws the preview away.
- **Reset** puts every value back to its starting value.

## Tips

- Right-drag in the **3D view**, away from the handles: left and right change **Sides**, up and down change **Segments**.

## Shaping it with a falloff

A falloff can taper, bend or twist the cone while you make it, as with the
Cube, Sphere and Cylinder. The falloff's weight decides how much each ring
of points changes: full where the weight is 1, not at all where it is 0.
More **Segments** give a smoother result.

1. With the cone tool open, pick a **Linear**, **Radial**, **Cylinder** or
   **Noise** falloff from the **Falloff** menu. The tool stays open.
2. Open the **Falloff** section in **Tool Properties** and set what the
   weight does, for example **Scale** 0% for a point at the full end.
3. Drag the falloff's handles to move it. The cone updates as you go, and
   as you change its own fields.
4. **Apply** or **Drop** to keep the shaped cone. **Undo** takes it back
   in one step.

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

You can also add a Cone as a step of a mesh's **Mesh Stack**: choose
**Cone** under **Create > Primitives** in the stack's **Add Operator**
list. It has the same properties as the tool, and the tool and the stack
step build exactly the same cone from the same values, so you can
change it later without rebuilding the mesh. If a falloff is on when you add it, the step keeps its own copy;
its **Falloff** rows work as above, and **Use Active Falloff** and **Clear
Falloff** copy one in or remove it. **Min** and **Max**
aren't there. Scripts
add it with `meshop.create type:prim.cone.item`.

With the step selected in the stack and no tool on, its handles show in the
**3D view**. Drag an arrow to move it along that axis, a plane square or the centre dot to move it across a plane, and a size square at its edge to resize it, and its falloff's handles to change the falloff. Each drag is one undo step. A right-drag anywhere else changes its counts, as with the tool: left and right for **Sides**, up and down for **Segments**.

## Related

- [Primitives](../interface/window.md): the **Toolbox** on the **Create** tab
- [Interface](../interface/window.md): the **Work Plane** and **Tool Properties**
- [Cylinder](cylinder.md)
