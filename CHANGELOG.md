# Changelog

Notable changes, newest first. The day-to-day build log is
[docs/PROGRESS.md](docs/PROGRESS.md).

## Unreleased

- Tools aren't modal (UIC-307, GitHub #36): with a tool on, Properties,
  the Items list, the Mesh Stack and N work. The tool's change so far is
  kept as its own step, the command runs, and the tool starts again.
- Dragged handles stay under the pointer at any depth and angle (GitHub
  #40): axis arrows and Free Move used one scale taken at the orbit
  centre.

- USD import brings animation: moving items, camera lens changes and
  light intensity and colour changes come in as keys, at the file's frame
  rate and range, so a USD export and re-import keeps the motion.

## 0.1.1 (2026-11-24)

- **Updates:** Operandi checks for a newer build each time it starts
  (Preferences > Defaults > Application > Updates; Help > Check for
  Updates). It follows the Test channel by default for now, so new test
  builds, made every night the code changes, are offered as they come
  out; switch to Release to hear only about releases.

- OpenPBR Surface 1.1.2 is Operandi's material model: every Material
  channel has an OpenPBR identity, and new channels cover coat colour,
  IOR, darkening and anisotropy, sheen colour and roughness, thin film,
  Abbe number, transmission scatter, thin walled and subsurface albedo.
  New materials start at OpenPBR's defaults (scenes with them save as
  native version 41).
- MaterialX import and export: File > Import MaterialX… and File > Export
  Material as MaterialX…; the Preset Browser lists `.mtlx` files.
- **Look change:** the Material's Dispersion, stored only before, now
  renders on see-through glass. Older scenes that set it look different.
