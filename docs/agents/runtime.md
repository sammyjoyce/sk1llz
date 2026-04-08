# Runtime

- Keep request validation symmetric across human and machine entrypoints: if a flag combination is invalid in the convenience surface, the raw `--request` JSON path must reject the same contradiction before planning or writing files.
- Treat `recommend from-path` as a bounded signal scan rather than a crawler: cap recursion depth, skip hidden/build/dependency trees, and prefer relevance over completeness so recommendations stay fast and useful on large repos.
- Treat explicit `--json` as a hard output contract, not a best-effort hint: commands that inherently emit non-JSON artifacts should fail with a usage error instead of mixing shell code or other raw bytes into a machine-readable workflow.
