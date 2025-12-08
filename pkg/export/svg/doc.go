/*
Package svg provides functionality to export Sruja architectures to SVG diagrams.

It supports various visual styles (Professional, C4, Minimal) and layout directions (TB, BT, LR, RL).
The layout engine uses a hierarchical cluster layout algorithm to position elements, optimizing for minimal edge crossings and balanced placement.

Key components:
  - Exporter: Main entry point for generating SVGs.
  - Layout: Internal layout engine for positioning nodes and edges.
  - Theme: Customizable visual themes.
*/
package svg
