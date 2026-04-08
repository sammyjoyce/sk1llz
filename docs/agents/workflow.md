# Workflow

- For any skill rewrite task, run a strict two-pass loop: first evaluate the current file against the 8 skill-judge dimensions, then perform external research on non-obvious failure modes before writing, because rewriting against known patterns without new signal produces low D1 and low transfer value.
- In skill-judge work, keep a decision matrix for constraints vs freedom (for example, philosophy-heavy edits can stay unconstrained, while rubric-driven rewrites require explicit section coverage and required triggers), then enforce that matrix while drafting to prevent accidental over- or under-prescription.
- Treat lookup tooling failures as signals: if `web_search_exa` fails on credentials, switch to web search immediately and continue with verifiable public sources, documenting only the fallback decision rather than halting the task.
- When a request targets a single file under `paradigms/` or `tools/`, capture only durable process knowledge in `docs/agents/<area>/guide.md` and avoid one-off diffs in top-level logs.
