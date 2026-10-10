# Responsive gallery

Pictures are SVG from [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot). Every glyph is one cell. Widths 1 through 100 are asserted in `tests/widths.rs`.

## Normal mode, one pane

Default theme colors.

### 20

![20 columns](shots/w20.svg)

### 40

![40 columns](shots/w40.svg)

### 80

![80 columns](screenshot.svg)

### 100

![100 columns](shots/w100.svg)

## Session mode

The word Session sits at the far left, before `Ctrl +`. The keys that follow are unmodified.

160 columns:

![Session mode at 160 columns](shots/session.svg)

## Tab mode

The word Tab sits at the far left, before `Ctrl +`.

160 columns:

![Tab mode at 160 columns](shots/tab.svg)

## Mouse over

Hovering a chip inverts that chip. At 80 columns the pointer is on `^N Resize`.

![Mouse over Resize at 80 columns](shots/hover.svg)

With two panes, `⌥←↓↑→` is four hit targets. Hovering `↓` inverts that arrow.

![Mouse over the down arrow at 160 columns](shots/hover-focus.svg)
