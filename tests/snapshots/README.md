Golden JSON snapshots for public output contracts.

Update only after an intentional public output change:

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli golden_snapshot
UPDATE_SNAPSHOTS=1 cargo test -p pine-wasm analysis_outputs_match_golden_snapshots
cargo test --workspace
```

Review the JSON diff before committing.
