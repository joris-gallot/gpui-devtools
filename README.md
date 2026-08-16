# gpui-devtools

[![CI](https://github.com/joris-gallot/gpui-devtools/actions/workflows/ci.yml/badge.svg)](https://github.com/joris-gallot/gpui-devtools/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/gpui-devtools.svg)](https://crates.io/crates/gpui-devtools)
[![License](https://img.shields.io/crates/l/gpui-devtools.svg)](LICENSE)

Inspect and debug [GPUI](https://gpui.rs) applications with an element picker, layout information, source locations, and style details.

The first release provides an element picker and inspector with:

- source locations and GPUI element IDs
- element bounds and content size
- raw `Div` style refinements
- a global toggle action and optional default keybinding

## Install

```toml
[features]
devtools = ["dep:gpui-devtools", "gpui/inspector"]

[dependencies]
gpui-devtools = { version = "0.1", optional = true }
```

Version 0.1 is compatible with GPUI 0.2.

## Use

```rust
app.run(|cx| {
    #[cfg(feature = "devtools")]
    gpui_devtools::init(cx);

    // Initialize the rest of the app.
});
```

The default shortcut is `cmd-alt-i` on macOS and `ctrl-alt-i` elsewhere. You can dispatch `gpui_devtools::ToggleInspector` yourself or customize installation:

```rust
gpui_devtools::init_with(
    gpui_devtools::Config::default().key_binding(None),
    cx,
);
```

The inspector relies only on GPUI's Apache-2.0 inspector hooks. Its implementation is original and does not include Zed's GPL-licensed inspector UI.

## Roadmap

- computed style view and temporary style editing
- render, layout and paint profiler
- repaint highlighting
- focus and keybinding inspector
- action and event timeline
- entity lifecycle diagnostics

## License

[MIT](LICENSE)
