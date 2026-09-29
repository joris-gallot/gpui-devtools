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

- `src/lib.rs`: public API, installation, inspector shell, and top-level rendering flow.
- `src/box_model.rs`: box model data extraction and rendering.
- `src/computed.rs`: computed/measured style panel.
- `src/copy.rs`: clipboard helpers and copy feedback state.
- `src/style_export.rs`: style summaries, override diffs, and Rust snippet exports.
- `src/styles.rs`: explicit style grouping, formatting, and style panel rendering.
- `src/temporary_edits.rs`: temporary style edit controls, state, and mutation logic.
- `src/ui.rs`: shared UI helpers used by inspector panels.
- `tests/inspector.rs`: GPUI interaction tests driven through the public API.
- `examples/basic/`: minimal standalone application for visual testing.

Keep the core crate independent from Zed UI and `gpui-kit`. The crate should stay framework-agnostic and avoid app-specific UI dependencies.

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

For release prep, also run:

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo publish --dry-run --allow-dirty
```

## Releases

- Before creating or editing a GitHub release, inspect recent releases with `gh release list` and `gh release view` and match the existing format.
- Release notes should usually use:
  - `## What's new`
  - `## Fixes` when applicable
  - `## Breaking changes` when applicable
  - `## Improvements` when applicable
  - a final compatibility sentence like `` `gpui-devtools` 0.5 targets `gpui-pre` 0.3. ``
- Do not include validation command output in release notes unless prior releases already do.
- For version releases, update `Cargo.toml`, `Cargo.lock`, and `README.md` install/compatibility docs before publishing.

## Code style

- Prefer small, typed public APIs.
- Keep framework-specific integrations outside the core crate.
- Avoid comments unless they explain why a non-obvious constraint exists.
- Do not commit or push changes.
