# gpui-devtools

[![CI](https://github.com/joris-gallot/gpui-devtools/actions/workflows/ci.yml/badge.svg)](https://github.com/joris-gallot/gpui-devtools/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/gpui-devtools.svg)](https://crates.io/crates/gpui-devtools)
[![License](https://img.shields.io/crates/l/gpui-devtools.svg)](LICENSE)

Inspect and debug [GPUI](https://gpui.rs) applications with an element picker, layout information, source locations, computed values, and style details.

![GPUI DevTools demo](https://raw.githubusercontent.com/joris-gallot/gpui-devtools/main/.github/assets/devtools.gif)

The inspector includes:

- an element picker
- source locations and GPUI element IDs with copy actions
- a box model view with element bounds, content size, margin, border, and padding
- a computed values panel with measured bounds, content size, explicit dimensions, spacing, opacity, visibility, and background
- temporary quick style edits for the selected `Div`, including presets, incremental size/spacing/opacity controls, background, visibility, reset, and copy/export actions
- grouped `Div` style refinements with compact spacing and color previews
- a global toggle action and optional default keybinding

## Install

```toml
[features]
devtools = ["dep:gpui-devtools", "gpui/inspector"]

[dependencies]
gpui = { package = "gpui-pre", version = "0.3" }
gpui-devtools = { version = "0.5", optional = true }
```

GPUI DevTools 0.5 targets [`gpui-pre`](https://crates.io/crates/gpui-pre) 0.3, the crates.io snapshot of Zed's GPUI also used by [gpui-kit](https://github.com/longbridge/gpui-kit). Your application must resolve to the same GPUI crate, otherwise Cargo builds two incompatible copies.

| gpui-devtools | GPUI |
| --- | --- |
| 0.5 | `gpui-pre` 0.3 |
| 0.4 | `gpui-pre` 0.3 |
| 0.3 | `gpui-pre` 0.3 |
| 0.2 | `gpui` 0.2 |

## Use

```rust
app.run(|cx| {
    // Initialize the rest of the app first.

    #[cfg(feature = "devtools")]
    gpui_devtools::init(cx);
});
```

Install GPUI DevTools after libraries that register their own inspector renderer.

The default shortcut is `cmd-alt-i` on macOS and `ctrl-alt-i` elsewhere. You can dispatch `gpui_devtools::ToggleInspector` yourself or customize installation:

```rust
gpui_devtools::init_with(
    gpui_devtools::Config::default().key_binding(None),
    cx,
);
```

## Roadmap

- deeper style inspection and live editing
- better element navigation, selection, and source discovery
- visual debugging helpers such as repaint and layout overlays
- focus, keybinding, action, and event inspection
- render, layout, and paint diagnostics
- entity and lifecycle debugging tools

## License

[MIT](LICENSE)
