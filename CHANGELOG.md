# Changelog

## 1.0.0 (2026-10-09)


### Features

* clickable buttons ([f31755a](https://github.com/fulldecent/zellij-status-bar-ng/commit/f31755adabd1df310af1750ec44efb75667a1dd6))
* initial implementation ([cc358ad](https://github.com/fulldecent/zellij-status-bar-ng/commit/cc358ade9987ddf5763fed4433536e6957d705c0))


### Bug Fixes

* allow to select permission prompt ([62610a8](https://github.com/fulldecent/zellij-status-bar-ng/commit/62610a871b152a18398de910f544ac73737b279a))

## Changelog

## Unreleased

- Gallery pictures for Session mode, Tab mode, a hovered Resize chip, and a hovered focus arrow.
- In-mode shortcuts drop the `^` caret. Session shows `L Layout Manager`, because the extra key is unmodified once the mode is active.
- Clicking Session chips such as Layout Manager launches the floating plugin instead of only leaving the mode.
- Active modes (Tab, Session, Pane, and the rest) put that name at the far left, before `Ctrl +`, so the following shortcuts read as that mode’s commands.
- Rebuild from the prior status-bar-ng tree on the zellij-plugin-template layout, reverse-DNS wasm names, and signed releases.
- README pictures are SVG from Zellij Plugin Snapshot. The PNG Chrome gallery is gone.
- Clickable shortcut chips with inverted hover. Letter combos hit one glyph; `^N Resize` is nine cells.
