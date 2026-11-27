<!-- help-ids: Action Center ; template: concept ; since: 0.1.0 -->
# Action Centers

An Action Center decides two things for **Move**, **Rotate**, **Scale** and **Transform**: where the handle sits (the center the tool turns and scales about) and which way its three arrows point (the axes your values are measured along). You choose one from the **Action Center** button in the row of modeling aids under the viewport.

The same choice also steers other tools that work about a center, such as **Flex** and **Smooth Shift**.

## Choose an Action Center

1. Click **Action Center** under the viewport.
2. Click a name in the list. The handle of the tool you are using moves and turns at once, and what you have already dragged is kept.

The top line of the menu shows the center and the axis in use. The **Center** and **Axis** submenus at the bottom change one of the two and keep the other.

| Name | Center | Axes |
| --- | --- | --- |
| **None** | Middle of the box around what the tool moves. | World |
| **Automatic** | Middle of the box around what the tool moves. | World |
| **Selection Border** | Middle of the selection's border (see below). | Follow the selection's shape |
| **Selection Center Auto Axis** | Middle of the box around the selection. | World |
| **Element** | The polygon, edge or point you click. | From what you clicked |
| **Screen** | Where you click in the view. | Face the view |
| **Selection** | Middle of the box around the selection. | Follow the selection's shape |
| **Origin** | The world origin. | World |
| **Local** | Each connected piece about its own middle. | Each piece's own |
| **Parent** | The parent item's position. | The parent item's |
| **Pivot** | The item's own position. | The item's own, turned with the item |
| **Pivot Center Parent Axis** | The item's own position. | The parent item's |

With nothing selected, "what the tool moves" is the whole mesh.

## Selection and Selection Border

Both give the handle axes that follow the shape of what you selected, so the arrows line up with your model instead of with the world:

- **X** (red) points along the direction the selection is longest in.
- **Z** (blue) points through the direction it is thinnest in. For a flat patch of polygons that is straight out of the surface.
- **Y** (green) is the direction left over, square to the other two.

A few examples: a flat patch gets Z along its normal and X along its longer side. A long strip gets X along the strip. A band of polygons around an arm or a pipe gets X along the arm when the band is longer than the arm is thick.

When two directions are equally long (a square patch, a round ring), the tie is broken toward the world X axis, so the handle does not spin to a random angle. A shape with no longer or shorter direction at all, such as a cube, a single point or nothing to measure, gets the World axes.

The two differ only in where the handle sits:

- **Selection** puts it in the middle of the box around the selection.
- **Selection Border** puts it in the middle of the border: the edges where your selected polygons meet polygons you did not select. Open edges at the rim of a mesh do not count. If the selection touches the rest of the mesh in two places, like a band around an arm, the handle sits halfway between the two. This is the center to use for bending a limb at a joint or turning a panel about its hinge.

**Selection Border** needs selected polygons with unselected neighbors. With edges or points selected, or when you select a whole mesh, it works like **Selection**.

Both work on any selection, whatever its shape.

## With Symmetry on

When **Symmetry** is on and your selection has a mirrored half, the **+** side is in charge:

- The handle sits on the **+** side, and for **Selection** and **Selection Border** its center and its axes come from the selected points on that side only. The mirrored half does not pull the arrows off line.
- The **+** side moves, turns and scales exactly as you drag or type.
- The other side does the mirror image: a move toward the mirror plane becomes a move toward it from the other side, and a turn goes the other way round.

If nothing on the **+** side is selected, the handle goes to the other side and **Selection** and **Selection Border** use the World axes.

**Origin**, **Parent** and **Pivot** keep their fixed centers when Symmetry is on.

## Element

Choose **Element**, then click a polygon, an edge or a point to put the handle on it. Your selection does not change. Until you click something, the handle is at the world origin with the World axes.

## Pivot

**Pivot** uses the item's own position and the item's own axes. If the item has been rotated, the arrows are rotated with it, so **Move** X slides the selection along the item's own X direction.

## Related

- [Interface](window.md)
- [Tool Properties](../panels/tool-properties.md)
- [Flex](../deform/flex.md)
