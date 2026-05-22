Workspace fixture ownership guide.

- `runtime/`: executable Pine behavior fixtures shared by runtime integration,
  CLI matrix, and conformance checks.
- `sema/`: semantic acceptance and rejection fixtures for compatibility reports.
- `syntax/`: parser fixtures that should not require semantic or runtime support.
- `realtime/`: forming-bar rollback fixtures for runtime integration tests.
- `profile/`: deterministic larger fixtures for runtime storage/profile gates.
- `conformance.tsv`: the fixture-backed feature matrix used by `pine-cli matrix`.

Keep cross-crate behavior fixtures here. Keep tiny helper edge cases in the
owning crate's module tests.
