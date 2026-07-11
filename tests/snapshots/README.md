Golden JSON snapshots for public output contracts.

Update only after an intentional public output change:

```text
UPDATE_SNAPSHOTS=1 cargo test -p pine-cli golden_snapshot
UPDATE_SNAPSHOTS=1 cargo test -p pine-wasm analysis_outputs_match_golden_snapshots
cargo test --workspace
```

Review the JSON diff before committing.

`scripts/check_host_parity.py` discovers every ordinary CLI runtime snapshot,
then verifies the explicit Python/WASM golden baseline in
`scripts/host_parity_required.txt`. The manifest is intentionally narrower than
the CLI registry: host suites keep representative public-contract coverage
without duplicating the CLI runner's fixture-specific input setup. Add a name to
the sorted manifest only after both host suites assert that golden snapshot.

At this stage the live gate discovers 695 registered CLI snapshots and verifies
418 manifest-required snapshots in both hosts. Registered snapshots must either
have no public-host assertion yet or be asserted by both hosts; a single-host
assertion fails the gate unless it has an explicit, reasoned exception. The
current exception set is empty. The script prints both live counts, so changing
either the registry or the paired-host baseline is an explicit, reviewable
change rather than a side effect of tuple formatting.
