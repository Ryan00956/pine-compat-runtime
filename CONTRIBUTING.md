# Contributing

This project is a clean-room Pine-compatible indicator runtime.

## Rules

- Do not copy TradingView source code, private APIs, proprietary data, UI, icons,
  branding, or error text.
- Prefer original fixtures written for this project.
- Include license and source metadata for non-original fixtures.
- Unsupported language behavior must produce diagnostics instead of panics.
- Keep host-specific adapters outside the core runtime crates.

## Development

Before submitting changes, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

