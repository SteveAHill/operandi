<!-- help-ids: Edge Bevel, edge.bevel, Edge Chamfer, edge.chamfer ; template: tool ; since: 0.1.0 -->
# Edge Bevel and Edge Chamfer

**Edge Bevel** rounds the selected edges off with new rows of polygons; **Edge Chamfer** cuts them flat.

![The top front edge of a 1 m cube rounded with Edge Bevel: Inset 0.15 m, Segments 6](../images/edge-bevel.png)

## How to use

1. Press **2** for Edges and select the edges.
2. Press **B** (Edge Bevel), or click **Edge Bevel** or **Edge Chamfer**. The handles show at once on the selection.
3. Drag the arrow, drag in the view, or type the **Inset**.
4. Press **Q**, or click **Drop Tool**, to keep the result. **Cancel** takes it back.

![The same edge cut flat with Edge Chamfer, 0.15 m](../images/edge-chamfer.png)

## Handles

- **Inset** (Edge Bevel): the arrow along the selection's average normal. Drag it to widen the bevel.
- **Mitering Offset** (Edge Bevel): the second arrow, across the strip. Drag it to add an edge beside the bevel on each side.
- A right-drag anywhere in the view changes **Segments**: right or up for more, left or down for fewer. A middle drag changes **Depth**.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Mode** | **Inset** measures along the sides; **Width** measures across the corner, so the sides move further on sharp corners. | Inset |
| **Inset** (or **Width**) | How far the bevel reaches from the edge. | 0 m |
| **Mitering Offset** | Edge Bevel: adds an edge this far beyond the bevel on each side, to hold its shape when smoothed. 0 adds none. | 0 m |
| **Segments** | How many rows of polygons run across the bevel; 1 is a flat strip. Edge Bevel and Edge Chamfer count the same way. | 1 |
| **Edge Shape** | **Round**, or **Square** and **Sharp** (two flat strips meeting at the original edge). | Round |
| **Depth** | 100% is the full round profile; lower values flatten it toward a chamfer. Needs **Segments** 2 or more. | 100% |
| **Loop Slide** | Edge Bevel. Off: points of the polygons around the bevel move out of its way, so it keeps its full width. On: nothing around it moves, and the bevel slides along the edges beside it and narrows where it has to, as at a corner that isn't square. | On |
| **Mitering Style** | Edge Bevel: how strips meet where three or more come together. **Default**: a rounded corner, its rows wrapping around in an even grid. **Neutral**: a rounded corner whose lines run straight up into it. **Uniform**: rows nest as smooth arches under a pointed arch. **Pinch**: rows meet in V seams under an arch pinched to a point. Needs **Segments** 2 or more. | Default |
| **Parallel** | Edge Chamfer: keeps each new edge the offset away from the original edge. | — |
| **Stop At Edge** | Edge Chamfer: side points stop at the far end of their edge instead of running past it. | — |
| **Auto Weld** | Edge Chamfer: merges new points that land on another point. | — |
| **Edge End** | Edge Chamfer. **Profile**: an inner end keeps its point. **Triangle Fan**: it slides along the edge beyond and closes with triangles. | Profile |
| **Use Material**, **Material Name** | Gives the new polygons a material. | Off |

The greyed rows (**Sharp**, **Coplanar** and the others below them) aren't available yet.

## Tips

- With **Loop Slide** off, the bevel keeps its full width round corners that aren't square, and a wide bevel pushes nearby points along their edges, or across the polygon beyond when they sit on a straight line, only as far as it needs. Corners of the solid stay where they are.
- Small rounded corners that a wide bevel would fold over close to a single point instead.

## Related

- [Edge tab](index.md)
- [Selection: loops and rings](../selection/loops.md)
- [Bevel (polygons)](../modeling/polygon-bevel.md)
- [Vertex Bevel](../vertex/vertex-bevel.md)
