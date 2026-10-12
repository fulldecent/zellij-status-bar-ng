# Changelog

## [1.1.0](https://github.com/fulldecent/zellij-status-bar-ng/compare/v1.0.0...v1.1.0) (2026-10-10)


### Features

* put the active mode name left of Ctrl + ([9c84be4](https://github.com/fulldecent/zellij-status-bar-ng/commit/9c84be4d9dc80e869f5242f5687cd055f3a94070))
* put the active mode name left of Ctrl + ([67f983f](https://github.com/fulldecent/zellij-status-bar-ng/commit/67f983f59e200244248728c766c6fb51dc889b90))


### Bug Fixes

* copy screenshot.svg to the repository root in the refresh hint ([b0a0a8f](https://github.com/fulldecent/zellij-status-bar-ng/commit/b0a0a8f42c99a3b8de1f12fdfd058e5f34ff7ba5))
* keep compact heading fallbacks off Normal and Locked ([ce83dfa](https://github.com/fulldecent/zellij-status-bar-ng/commit/ce83dfa76a6f9277ec30fa7ad0c1285303a836b7))
* keep the mode heading on narrow panes ([fea2663](https://github.com/fulldecent/zellij-status-bar-ng/commit/fea26636bf916e205d97f740997738418466068a))
* unchorded in-mode keys and floating plugin clicks ([9b5ad0a](https://github.com/fulldecent/zellij-status-bar-ng/commit/9b5ad0a6f63f8fe544531e6530c80a0e4e405acc))
* unchorded in-mode keys and floating plugin clicks ([def16c3](https://github.com/fulldecent/zellij-status-bar-ng/commit/def16c324d6ea03ac58e04bfbeafef374e7d94ea))

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
