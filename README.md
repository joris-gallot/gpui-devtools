# gpui-devtools

MIT-licensed, framework-agnostic developer tools for [GPUI](https://gpui.rs) applications.

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
gpui-devtools = { git = "https://github.com/joris-gallot/gpui-devtools", optional = true }
```

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
