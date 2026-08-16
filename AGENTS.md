# GPUI DevTools Agent Guide

## Project

`gpui-devtools` is an MIT-licensed, framework-agnostic developer toolkit for GPUI applications.

The goal is to provide Chrome DevTools-like inspection and diagnostics for GPUI without depending on application-specific UI frameworks.

## Licensing

- Keep all project code MIT-compatible.
- GPUI public APIs are Apache-2.0 and may be used normally.
- Never copy or adapt code from Zed's `crates/inspector_ui`, which is GPL-3.0-or-later.
- Implement features originally from public API documentation and observed behavior.
- Avoid adding GPL or AGPL dependencies.
- Check the dependency tree and licenses before adding dependencies.

## Architecture

- `src/lib.rs`: public API and inspector implementation.
- `examples/basic/`: minimal standalone application for visual testing.

Keep the core crate independent from Zed UI and `gpui-component`.

## Development

- Use current GPUI documentation and inspect the exact installed GPUI source when API details matter.
- Keep the public setup API simple, ideally `gpui_devtools::init(cx)`.
- Make development tooling opt-in so applications do not ship it accidentally.
- Add tests for every feature and bug fix.
- Update `README.md` when public APIs, setup, shortcuts, or capabilities change.
- Add or update the basic example for features that need visual validation.

## Validation

Run before finishing:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo check --workspace
cargo package --allow-dirty --no-verify
```

## Code style

- Prefer small, typed public APIs.
- Keep framework-specific integrations outside the core crate.
- Avoid comments unless they explain why a non-obvious constraint exists.
- Do not commit or push changes.
