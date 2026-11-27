<!-- help-ids: Select All, Invert, Connected, Invert Connected, Grow, Shrink, Select More, Select Less, Convert to Vertices, Convert to Edges, Convert to Polygons, Boundary, Select Symmetrical, Lazy Selection, select.all, select.invert, select.connect, select.expand, select.contract, select.more, select.less, select.convert, select.boundary, select.symmetrical, select.lazyState ; template: panel ; since: 0.1.0 -->
# Select tab

The **Select** tool tab holds every command that changes the selection. Each button runs once on the current selection; its key is shown on the button.

![The Select tab, and one polygon of a sphere grown twice](../images/select-grow.png)

## Changing the selection

| Button | Key | What it does |
|---|---|---|
| **All** | **Alt+Shift+A** | Selects everything in the current mode. |
| **Invert** | **[** | Selects what wasn't selected, and drops what was. |
| **Connected** | **]** | Adds everything connected to the selection: the whole piece. |
| **Invert Connected** |  | Inverts the selection only inside the pieces it touches. |
| **Grow** | **Shift+Up Arrow** | Adds every neighbour of the selection, all the way round. |
| **Shrink** | **Shift+Down Arrow** | Takes away the selection's outer edge. |
| **More** | **Up Arrow** | Continues the pattern: adds the next element along the line through the last two you picked. |
| **Less** | **Down Arrow** | Takes back the last element **More** added. |
| Convert buttons | **Alt+1**, **Alt+2**, **Alt+3** | Turn the selection into points, edges or polygons. |
| **Boundary** |  | Selects the edge round the outside of the selection. |
| **Symmetrical** |  | Selects the mirror partners of the selection (with **Symmetry** on). |
| **Lazy Selection** |  | On: a click picks the nearest point or edge from further away. |

**Shift+Up Arrow** and **Shift+Down Arrow** grow or shrink the selection while **Move**, **Rotate**, **Scale** or **Transform** is active; the tool stays on, and the next drag moves the new selection.

The rest of the tab has pages of its own: [Loops and rings](loops.md), [Between and shortest path](between-and-paths.md), [Select by shape](select-by-shape.md) and [Selection sets](selection-sets.md).

Two settings sit in the tab too: the lasso's shape (freeform, rectangle, circle or ellipse), and which polygon tag the **Materials** mode selects by.

## Related

- [Selection modes](selection-modes.md)
- [Loops and rings](loops.md)
