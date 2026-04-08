# CLI Area Guide

- Treat `cli/` as both a human CLI and an agent surface: every command redesign should keep stdout machine-clean, move diagnostics to stderr, and add structured output, dry-run paths, and discoverable help before polishing text formatting.
- Keep the command tree narrow and noun-verb shaped: `catalog`, `install`, `remove`, `recommend`, `env`, `describe`, and `completions` are the stable top-level surfaces; mutations go through explicit `plan` and `apply` subcommands instead of one-shot write commands.
- Repo-local scope is anchored to the repo root only: use `<repo-root>/.claude/skills/` when that directory exists, fall back to `~/.claude/skills/` otherwise, and never walk past the repo boundary to reuse an unrelated ancestor `.claude/`.
- Preserve the agent surfaces as first-class behavior, not documentation sugar: `--json` must stay consistent across commands, `describe` is the machine-readable schema entry point, commands that emit shell code such as `completions` should reject explicit JSON requests, and install/remove must keep raw `--request` JSON validation aligned with the equivalent flag path.
- Field masks on wrapped result types must still expose nested skill fields directly, so selectors like `id,name,description` remain portable across `catalog list`, `catalog search`, and `recommend` without forcing callers to special-case wrapper JSON.
