<!-- help-ids: edge.bridge.item, poly.loopSlice.item, poly.thicken.item, poly.setType.meshop.item, edge.slide.item, setvmap.subd.item, hardedge.meshop.item, hardedgeconvert.meshop.item, chamfer.edit.item, pmodel.partTag.item, assignselectionset.meshop.item, curveconvert.meshop.item, curveWinding.meshop.item, curvefill, curveBoolean.meshop.item, pmodel.edgeToCurve.item, cRebuild.mOp, curve.sweep, radial.sweep.item, prim.text.item, prim.arc.item, uv.create.item, uv.relax.item, uv.pelt.item, uvflip.meshop.item, uvconvert.meshop.item, setvmap.create.item, setvmap.weight.item, setvmap.flood.item, setvmap.rgb.item, setvmap.seam.item, setvmap.vnorm.item, pmodel.geodesic.mOp, Create UV Map, UV Relax, UV Unwrap and Relax, UV Flip, Convert UV to Mesh, Create Vertex Map, Set Weight, Flood Weight, Set RGB, Set Seam, Set Vertex Normals, Geodesic Weight, Curve Convert, Curve Winding Aligner, Curve Fill, Curve Boolean, Curve Rebuild, Curve Sweep, Assign Selection Set, Set Polygon Type, Hard Edge, Hard Edge Convert, Part Tag ; template: concept ; since: 0.1.2 -->
# Tool steps in the Mesh Stack

Many modeling tools can also be a step in a mesh's **Mesh Stack**, so you
can change their values later without rebuilding the mesh. A step runs
the same code as its tool, so the tool and the step give the same result
from the same values. Add one from the stack's **Add Operator** list. Its
values show in **Properties**, and its **Selection** section says which
part of the mesh it works on (see
[Mesh Stack selections](mesh-stack-selections.md)).

With nothing in its **Selection**, a step works on the whole mesh: every
polygon, or every edge for the edge steps.

## The steps

| Step | Folder | Works on | What it does |
|---|---|---|---|
| **Bridge** | Edit | Edges | Joins two edge runs or loops with new polygons, as [Bridge](../duplicate/bridge.md). With nothing selected it bridges the mesh's open borders, so two loose pieces with one open border each join up. |
| **Loop Slice** | Edit | Edges | Cuts loops across the ring of each chosen edge, as [Loop Slice](../edit/slice-tools.md). With a **Count** above 1 the loops are spaced evenly and **Position** is set aside. |
| **Hard Edge** | Edit | Edges | Marks the chosen edges hard (or, with **Hardness** set to **Soft**, soft) and rebuilds the vertex normals. **Mode: By Angle** takes only the chosen edges sharper than **Angle**. **Clear Other Hard Edges** softens the rest. |
| **Hard Edge Convert** | Edit | Whole mesh | Turns every edge where the shading splits into a hard edge. It can also remove the normal map and the smoothing groups. |
| **Edge Slide** | Edge | Edges | Slides the chosen edges along their neighboring edges, as [Slide](../edit/slide.md), by **Percent** or by **Distance**. |
| **Chamfer Edit** | Edge | Polygons | Reworks a chamfer or bevel made lower in the stack: pick its strip polygons, then change **Offset**, **Segments** and **Depth**. |
| **Thicken** | Polygon | Polygons | [Smooth Shift](../polygon/smooth-shift.md) with **Thicken** on: a sheet gets a back side and walls. **Offset** sets the thickness. |
| **Set Polygon Type** | Polygon | Polygons | Makes the chosen polygons plain faces, **Subdivision** polygons or **Catmull-Clark** polygons. It sets the type rather than switching it, so it gives the same result every time. |
| **Part Tag** | Polygon | Polygons | Gives the chosen polygons a part name, as [Set Part](../polygon/part-material.md). |
| **Assign Selection Set** | Selection | Polygons | Stores the chosen polygons as a selection set called **Name**, so steps above it can pick them with **Select by Selection Set**. |
| **Set Edge Weight** | Vertex Map | Edges | Gives the chosen edges a subdivision weight, as [Edge Weight](../edge/edge-weight.md). **0%** removes their weights. |
| **Radial Sweep** | Duplicate | Edges | Spins a profile round an axis into a surface, as [Radial Sweep](../duplicate/radial-sweep.md). With nothing selected the whole mesh is the profile, and its lines and curves are used up. |
| **Text** | Create / Primitives | Adds | Adds the letters of **Text** as flat polygons, as the [Text](../modeling/text.md) tool. Empty text adds nothing. |
| **Arc** | Create / Primitives | Adds | Adds an arc-shaped polygon round **Center**, starting toward **Start** and turning **Angle** toward **End**, as the [Arc](../modeling/arc.md) tool. |

## Curve steps

These steps work on the curves in the mesh. Polygons among the chosen
elements pass through unchanged, and with nothing selected a step uses
every curve.

| Step | Works on | What it does |
|---|---|---|
| **Curve Sweep** | Curves | Puts a round tube of diameter **Size** along each curve. **Steps** sets how many rings follow the curve (or **Auto** adds them where it bends), **Rotation** rolls the tube and **End Caps** closes its ends. **Curve Index** picks curves by their order, for example `0,2`. The curves stay. |
| **Curve Fill** | Curves | Fills each closed curve with a flat polygon, as [Make Curve Fill](../curve/curve-commands.md). **Delete Source Curve** (on at first) removes the curves; **Flip Source Curves** turns the fills over. |
| **Curve Boolean** | Curves | Cuts the chosen closed curves against the others: **Slice**, **Union**, **Intersect** or **Subtract**. With nothing selected the curves cut each other. |
| **Edges to Curves** | Edges | Turns the chosen edges into curves, joined end to end (**Continuous**) or one per edge (**Separate**), as lines, Curves or Beziers. With nothing selected it traces the open borders. **Delete Geometry** removes the polygons the edges came from. |
| **Curve Rebuild** | Curves | Rebuilds each curve through evenly spaced points: **Point Count** points, or one every **Point Spacing**. |
| **Curve Convert** | Curves | Converts curves of one type (**From**) to another (**To**): Curve, Bezier or B-Spline, as [Convert Curves](../curve/offset-convert.md). It starts as Curve to Bezier. |
| **Curve Winding Aligner** | Curves | Makes every closed curve run the same way round, as [Align Winding](../curve/curve-commands.md). **Merge Curves** first joins open curves that meet end to end; **Reverse** turns them all the other way. |

## UV and vertex map steps

These steps write into a mesh's UV maps and vertex maps. A step names its
map in **UV Map** or **Map Name**; a map that doesn't exist yet is made.

| Step | Works on | What it does |
|---|---|---|
| **Create UV Map** | Polygons | Projects the chosen polygons into the map: Planar, Cylindrical, Spherical, Atlas, Barycentric, Atlas 2 or Cubic. **Settings: Automatic** fits the shape to the polygons; **Manual** uses **Center**, **Size** and **Rotation**. |
| **UV Relax** | Polygons | Relaxes the chosen polygons' UVs, as the UV Relax tool. **Iterations** starts at 0, which leaves the UVs alone, so raise it to see a change. Polygons not chosen stay still. |
| **UV Unwrap and Relax** | Polygons | A UV Unwrap followed by a UV Relax, each switched on or off with its own checkbox and with its own **Iterations**. |
| **UV Flip** | Polygons | Mirrors the chosen polygons' UVs in U or V about their middle, as [Flip UVs](../uv/flip-uvs.md). **Flip per Polygon** flips each polygon on its own. |
| **Convert UV to Mesh** | Polygons | Replaces the mesh with its flat UV layout: U along X, V along Y. |
| **Create Vertex Map** | Whole mesh | Adds an empty map called **Map Name**: Weight, UV, Morph, RGB or RGBA. |
| **Set Weight** | Points | Gives the chosen points **Map Value** in a weight map, or adds it with **Additive**. |
| **Flood Weight** | Points | Sets every chosen point to 100% (**High**) or 0% (**Low**). |
| **Set RGB** | Points | Gives the chosen points a colour in a colour map; **Map Type: RGBA** uses **Value A** too. |
| **Geodesic Weight** | Points | Makes a weight map of each point's distance from the chosen points, measured along the edges. **Normalize** makes the farthest point 100%; **Invert** turns it round. |
| **Set Seam** | Edges | Adds the chosen edges to a UV seam map, so a UV Unwrap above can cut there. |
| **Set Vertex Normals** | Points | Rebuilds the vertex normal map on the chosen points, splitting at hard edges. |

A step that can't run (for example **Bridge** on a closed mesh with no
open border) turns its row red and passes the mesh below through
unchanged, so the rest of the stack still works.

## Related

- [Mesh Stack selections](mesh-stack-selections.md)
- [Properties panel](properties.md)
