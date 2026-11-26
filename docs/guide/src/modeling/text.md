<!-- help-ids: Text, prim.text ; template: tool ; since: 0.1.0 -->
# Text

**Text** turns words you type into flat polygons, in the font you pick.

![The word Operandi made with Text, with its Tool Properties](../images/text.png)

## How to use

1. Make sure the mesh you want to add to is selected in the **Items list**.
2. Click **Text** in the **Primitives** section of the **Create** tool tab.
3. Type your words in the **Text** field of **Tool Properties**.
4. Drag in the **3D view** to move the text, or drag one of the arrows at its
   centre to move it along just X, Y or Z.
5. Click **Drop** to keep it as polygons, or **Apply** to keep it and type
   more.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Text** | The words to make. | empty |
| **Font** | The typeface. | Ubuntu Light |
| **Size** | The height of the letters. | 1 m |
| **Tracking** | Extra space between letters. | 0 m |
| **Position** | Where the text starts (X, Y, Z). | 0 m |
| **Location** | Which part of the letters sits on **Position**: **Top**, **Middle** or **Bottom**. | Bottom |
| **Justification** | Which end of the line sits on **Position**: **Left**, **Center** or **Right**. | Left |
| **Axis** | The axis the text faces along. | Z |
| **Flip** | Mirrors the text so it reads from the other side. | Off |
| **Make UVs** | Gives the letters a UV layout. | Off |

- **Apply** adds the text to the mesh and keeps the tool, so you can
  make another.
- **Drop** adds it and closes the tool.
- **Cancel** throws the preview away.
- **Reset** puts every value back to its starting value.

## Tips

- Once dropped, text is ordinary polygons: you can't edit it as text
  again.
- With **Text** empty, **Drop** adds nothing.

## Related

- [Primitives](../interface/window.md): the **Toolbox** on the **Create** tab
- [Interface](../interface/window.md): the **Work Plane** and **Tool Properties**
