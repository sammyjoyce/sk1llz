# CLAUDE.md⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍‌‌‌‌‌​​‌‍​‌​​​​​​‍​‌‌​‌‌‌​‍‌​‌‌‌‌​‌‍‌​‌‌​‌‌​‍​​​‌​‌‌‌‍​​​​‌​‌​‍‌​‌‌​​​​⁠‍⁠

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

`sk1llz` is two things in one repo:

1. **A curated library of "skills"** — markdown files encoding the mental models, principles, and anti-patterns of legendary engineers and engineering cultures. Each skill is a directory containing `SKILL.md` (required) and optional supporting files.
2. **A Rust CLI** (`cli/`) that acts as a package manager for those skills — `sk1llz catalog`, `sk1llz install`, `sk1llz recommend`, `sk1llz env`.

Plus a second supporting Rust binary in `tools/sk1llz-dna/` that injects invisible zero-width-character fingerprints into every committed `.md`.

## Layout

```
cli/                  Rust CLI (sk1llz binary). Single main.rs, ~3100 lines.
tools/sk1llz-dna/     Independent Rust binary for steganographic .md fingerprinting.
scripts/              Python + shell automation (manifest, tags, elevation, install).
meta/                 skill-template/, validation/validate.py.
docs/agents/          Durable process knowledge for agents — see "Agent notes" below.
languages/ paradigms/ domains/ organizations/ specialists/
                      The five skill roots. Each leaf is `<engineer>/SKILL.md`.
skills.json           Generated manifest — source of truth for the CLI. Never hand-edit.
flake.nix             Nix dev shell (rust, cargo, clippy, rustfmt, python+pyyaml, openssl).
```

## Commands

### CLI (`cli/`)
```bash
cd cli
cargo check                       # fast typecheck
cargo build                       # debug build
cargo build --release             # release build
cargo test                        # all tests
cargo test <name>                 # single test by substring
cargo clippy -- -D warnings       # CI lint gate
cargo fmt --all -- --check        # CI format gate
make install PREFIX=~/.local      # install binary to $PREFIX/bin
```

### DNA tool (`tools/sk1llz-dna/`) — separate Cargo project
```bash
cd tools/sk1llz-dna
cargo test
cargo run --release -- inject .   # stamp all unstamped .md (CI does this on push)
cargo run --release -- verify .   # exit 1 if any .md missing/invalid stamp
cargo run --release -- decode <file.md>
```

### Manifest + skill validation
```bash
python3 scripts/generate_manifest.py          # regenerate skills.json
python3 meta/validation/validate.py           # lint frontmatter + manifest consistency
```

### Nix dev shell
```bash
nix develop                       # provides rust toolchain, python+pyyaml, openssl, pkg-config
```

## Gotchas (read before editing)

- **`skills.json` is generated.** Never edit by hand. Run `python3 scripts/generate_manifest.py` after adding/renaming/removing any `SKILL.md`. CI (`.github/workflows/update-manifest.yml`) will auto-regenerate and push otherwise, and the `validate-manifest` CI job fails if the committed file drifts from what the generator produces (ignoring the `generated_at` timestamp).
- **Zero-width characters in markdown are intentional.** Every committed `.md` is stamped by `tools/sk1llz-dna` via the `dna-stamp.yml` workflow. Do not strip the garbled-looking characters near headings — they encode an origin/timestamp/path fingerprint. If a diff shows them appearing/disappearing, that's the auto-stamp workflow; leave it alone.
- **`cli/` and `tools/sk1llz-dna/` are independent Cargo projects,** not a workspace. Each has its own `Cargo.lock` and `target/`. Run cargo commands inside the relevant directory.
- **Skill discovery is path-driven.** `scripts/generate_manifest.py` walks from the repo root looking for `SKILL.md`, and derives `category` from the first path component and `subcategory` from the second. New skills must live under one of the five skill roots (`languages/`, `paradigms/`, `domains/`, `organizations/`, `specialists/`) or they will not appear in the manifest. `meta/skill-template/` is skipped on purpose.
- **Frontmatter contract.** Every `SKILL.md` needs YAML frontmatter (`---` fenced) with at minimum `name` and `description`. `meta/validation/validate.py` enforces this and is gated in CI. `name` should be lowercase with hyphens and ≤64 chars; the generator also extracts `tags` (string or list) and normalizes hyphens to underscores.
- **OpenSSL dependency.** `cli/Cargo.toml` uses `reqwest` with `rustls-tls` (no system OpenSSL needed for TLS), but `openssl`, `pkg-config`, and `openssl-sys` are still provided by `flake.nix` for transitive builds. If `cargo check` fails on `openssl-sys` outside the Nix shell, enter `nix develop` or install the system `libssl-dev` + `pkg-config` — don't work around it by patching dependency features.
- **Skill content format is strict.** See `CONTRIBUTING.md` for the required `SKILL.md` structure (Overview, Core Philosophy, Design Principles, Always/Never/Prefer, Code Patterns, Mental Model). The `scripts/elevate-skills.sh` prompt encodes the stricter 8-dimension `skill-judge` rubric that content is expected to score on.

## Agent notes location

Per `docs/agents/workflow.md`, durable process knowledge for agents working in this repo (two-pass skill rewrite loop, decision matrices, runtime constraints, recovery from tool failures) lives in `docs/agents/*.md`, **not** in top-level logs or one-off scratch files. When you learn something reusable about operating in this repo, append a terse note to the relevant file in `docs/agents/`:

- `docs/agents/workflow.md` — process rules and decision matrices
- `docs/agents/runtime.md` — runtime/validation invariants (e.g. CLI surface symmetry, JSON output contract)
- `docs/agents/troubleshooting.md` — recovery recipes for tool/env failures

## CI gates

`.github/workflows/ci.yml` runs on every PR: `cargo check`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, cross-platform `cargo build --release` (linux/macos/windows), manifest-regeneration diff check, skill frontmatter validation, and the `sk1llz-dna` tool's own fmt/clippy/test. All must pass. Locally, the fast pre-push check is: `cd cli && cargo fmt --all -- --check && cargo clippy -- -D warnings && cargo test` plus `python3 scripts/generate_manifest.py && python3 meta/validation/validate.py`.
