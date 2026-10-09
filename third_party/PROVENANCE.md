# Stock status-bar sources

The `.rs` files in this directory are copied from Zellij tag `v0.45.1`:

<https://github.com/zellij-org/zellij/tree/v0.45.1/default-plugins/status-bar>

Zellij is MIT licensed. These files are the reference for `src/stock_line.rs`.

They are not part of the build. The upstream plugin is a workspace member (`zellij-tile` and `zellij-tile-utils` are path dependencies) and does not compile in this repository. `render_stock_status_line` follows `render_common_modifiers` and `long_mode_shortcut` / the shortened key ribbon for `InputMode::Normal` only.
