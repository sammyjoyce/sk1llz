# CLAUDE.md

Guidance for AI assistants (Claude Code and similar) working in this repository.

## What this repo is

`sk1llz` is a curated library of **AI coding skills** that encode the philosophies, heuristics, and mental models of legendary software engineers. It is consumed by AI agents (Claude, Cursor, Windsurf, etc.) so they can "think like" a specific engineer when working on a task.

The repository is three things at once:

1. **A knowledge base** — markdown skill files under `languages/`, `paradigms/`, `domains/`, `organizations/`, and `specialists/`.
2. **A package manager** — a Rust CLI in `cli/` that lists, searches, and installs skills into `~/.claude/skills/` or `.claude/skills/`.
3. **Tooling** — a Python manifest generator/validator and a Rust "DNA" tool that stamps markdown with steganographic fingerprints.

The machine-readable source of truth for all skills is [`skills.json`](skills.json), which is **auto-generated** from disk by `scripts/generate_manifest.py`.

## Repository layout

```
sk1llz/
├── languages/           # Skills organized by programming language
│   ├── c/ cpp/ go/ javascript/ python/ rust/ zig/
│   └── <lang>/<engineer>/SKILL.md
├── paradigms/           # distributed/ functional/ systems/
├── domains/             # api-design/ cli-design/ databases/ geospatial/
│                        # networking/ problem-solving/ search/ security/
│                        # systems-architecture/ testing/ trading/
├── organizations/       # google/ netflix/ jane-street/ cloudflare/ ...
├── specialists/         # security/forensics-team/ (cross-cutting teams)
├── meta/
│   ├── skill-template/SKILL.md    # Canonical template for new skills
│   └── validation/validate.py     # Structural validator
├── scripts/
│   ├── generate_manifest.py       # Regenerates skills.json from disk
│   └── enrich_tags.py             # Batch-adds tags to existing SKILL.md files
├── cli/                 # Rust CLI (the `sk1llz` binary) — clap, reqwest
│   ├── Cargo.toml
│   ├── Makefile
│   └── src/main.rs
├── tools/sk1llz-dna/    # Rust tool — zero-width-char fingerprinting of .md
├── .github/workflows/
│   ├── ci.yml                  # check, fmt, clippy, test, build, validate
│   ├── update-manifest.yml     # auto-regenerates skills.json on push
│   ├── dna-stamp.yml           # auto-stamps .md files on push
│   └── release.yml             # tags → multi-target binary release
├── skills.json          # GENERATED — do not hand-edit
├── README.md
└── CONTRIBUTING.md
```

## The SKILL.md format

Every skill lives at `<category>/<subcategory>/<engineer>/SKILL.md`. Every SKILL.md **must** begin with YAML frontmatter — the validator (`meta/validation/validate.py`) will fail CI otherwise.

```markdown
---
name: <lastname>-<brief-descriptor>
description: <≤200 chars, starts with "Write/Design/... in the style of ...". Explain when to use this skill.>
tags: comma, separated, lowercase-hyphenated, keywords
---

# <Full Name> Style Guide

## Overview
Brief bio and why this engineer matters.

## Core Philosophy
2–3 direct quotes or short paragraphs capturing their thinking.

## Design Principles
1. **Principle Name**: Explanation
2. ...

## When Writing Code
### Always
- ...
### Never
- ...
### Prefer
- X over Y because Z

## Code Patterns
BAD / GOOD comparisons with concrete snippets.

## Mental Model
How they approach problems.

## Additional Resources
Links to philosophy.md, references.md, patterns/, anti-patterns/, examples/.
```

Required frontmatter fields (enforced by `validate.py`): **`name`** and **`description`**. `tags` is strongly recommended — it is consumed by the CLI search and the manifest generator.

### Optional sibling files inside a skill directory

- `philosophy.md` — deep dive into mental models
- `references.md` — books, papers, talks
- `patterns/` — canonical code patterns
- `anti-patterns/` — what the engineer advises against
- `examples/` — worked examples

The manifest's `files` array for a skill is auto-populated from anything at the top level of the skill directory (see `get_skill_files` in `scripts/generate_manifest.py`).

## Naming conventions

- **Directories**: lowercase, hyphenated (`jane-street`, `systems-architecture`, `beck-tdd`).
- **Skill `name`** (frontmatter): `<lastname>-<descriptor>`, all lowercase, hyphenated (e.g. `matsakis-ownership-mastery`, `lamport-distributed-systems`).
- **Tags**: comma-separated in the YAML, lowercase, hyphenated. The manifest generator normalizes hyphens to underscores (`design-patterns` → `design_patterns`) when flattening into `skills.json`.
- **Descriptions**: ≤200 chars, imperative ("Write …", "Design …"), and should state **when** to apply the skill so AI agents can route to it.

## Skill categories & the `skills.json` manifest

`skills.json` is produced by `scripts/generate_manifest.py`, which walks the repo with `rglob("SKILL.md")`, skips `meta/skill-template/`, and emits one entry per skill:

```json
{
  "id": "<frontmatter name, or dir name>",
  "name": "<engineer directory name>",
  "description": "<frontmatter description>",
  "category": "<first path segment: languages|paradigms|domains|organizations|specialists>",
  "subcategory": "<second path segment, e.g. 'python'>",
  "path": "languages/python/vanrossum",
  "files": ["SKILL.md", "philosophy.md", ...],
  "tags": [...]
}
```

**Do NOT hand-edit `skills.json`.** Regenerate it:

```bash
python3 scripts/generate_manifest.py
```

CI validates that a regenerated `skills.json` is byte-identical to the committed one (ignoring the `generated_at` timestamp) — see `.github/workflows/ci.yml` job `validate-manifest`. A push to `master`/`main` that modifies any `SKILL.md` triggers `update-manifest.yml`, which regenerates and auto-commits `skills.json` on your behalf.

Note: the validator (`meta/validation/validate.py`) walks only `languages/`, `domains/`, `paradigms/`, `organizations/` (see `SKILL_ROOTS`). The manifest generator uses `rglob` and so additionally picks up `specialists/`. If you add a new top-level skill root, update **both**.

## Common development workflows

### Add a new skill

1. Pick the right home: `languages/<lang>/`, `paradigms/<paradigm>/`, `domains/<domain>/`, or `organizations/<org>/`.
2. Create `<root>/<subcategory>/<engineer>/SKILL.md` based on `meta/skill-template/SKILL.md`.
3. Fill in `name`, `description`, and `tags` in the frontmatter. Follow the structure in [CONTRIBUTING.md](CONTRIBUTING.md).
4. Regenerate the manifest:
   ```bash
   python3 scripts/generate_manifest.py
   ```
5. Validate:
   ```bash
   python3 meta/validation/validate.py
   ```
6. Commit both the new skill files **and** the updated `skills.json`.

### Edit an existing skill

Same as above — any change to a tracked `SKILL.md` means `skills.json` needs to be regenerated (CI will fail `validate-manifest` otherwise).

### Bulk-add tags to existing skills

`scripts/enrich_tags.py` contains a hand-maintained `TAG_MAP` from skill `name` → comma-separated tags. It will only inject tags into a skill that doesn't already have a `tags:` line. After running it, regenerate the manifest.

### Build / run the CLI (`cli/`)

Rust 2021, uses `clap` derive, `reqwest` (blocking), `serde`, `fuzzy-matcher`, `indicatif`. Release profile uses `strip`, `lto`, `codegen-units = 1`.

```bash
cd cli
make build          # cargo build
make release        # cargo build --release
make install        # installs to $PREFIX/bin (default ~/.local/bin)
make check          # cargo check
make fmt            # cargo fmt
make clippy         # cargo clippy -- -D warnings
make test           # cargo test
```

The CLI reads its manifest from `$SKILLZ_MANIFEST_URL` (default: `https://raw.githubusercontent.com/copyleftdev/sk1llz/master/skills.json`) and fetches raw files from `$SKILLZ_RAW_BASE_URL`. Install-location resolution: if `./.claude/` exists in CWD, skills go to `./.claude/skills/`; otherwise `~/.claude/skills/`. See `sk1llz where` and `sk1llz install --global`.

### Build / run the DNA tool (`tools/sk1llz-dna/`)

A Rust tool that embeds steganographic zero-width-character fingerprints into every markdown file. Uses a 23-byte payload (`SK1L` magic + version + origin hash + timestamp + path hash + checksum). The `dna-stamp.yml` workflow runs `sk1llz-dna inject .` on every push that touches `**/*.md` and auto-commits the result with `[skip ci]`.

```bash
cd tools/sk1llz-dna
cargo build --release
./target/release/sk1llz-dna inject . --dry-run   # preview
./target/release/sk1llz-dna verify .             # CI check
./target/release/sk1llz-dna decode path/to/file.md
```

You will see sequences of invisible characters (zero-width-space, zero-width-non-joiner, zero-width-joiner, word-joiner) in markdown files — **do not strip them**. They are load-bearing for provenance tracking and will be re-injected on the next push if removed. If an edit accidentally breaks a fingerprint, `dna-stamp.yml` will heal it automatically on push to `master`/`main`.

## CI/CD

`.github/workflows/ci.yml` runs on every push/PR to `master`/`main` and performs:

| Job | What it does |
|-----|--------------|
| `check` | `cargo check` in `cli/` |
| `fmt` | `cargo fmt --all -- --check` in `cli/` |
| `clippy` | `cargo clippy -- -D warnings` in `cli/` |
| `test` | `cargo test` in `cli/` |
| `build` | Release build matrix: ubuntu / macOS / windows |
| `validate-manifest` | Regenerates `skills.json` and fails if it differs |
| `validate-skills` | Runs `meta/validation/validate.py` |
| `dna-check` | `fmt`, `clippy`, `test` for `tools/sk1llz-dna` |

**Before pushing**, at minimum run:

```bash
(cd cli && cargo fmt --all && cargo clippy -- -D warnings && cargo test)
(cd tools/sk1llz-dna && cargo fmt --all && cargo clippy -- -D warnings && cargo test)
python3 scripts/generate_manifest.py
python3 meta/validation/validate.py
```

Other workflows:

- **`update-manifest.yml`** — auto-regenerates `skills.json` on push to master when any `SKILL.md` or the generator script changes.
- **`dna-stamp.yml`** — auto-stamps markdown with DNA fingerprints on push to master.
- **`release.yml`** — on `v*` tags, builds binaries for 6 targets (linux gnu/musl x86_64 + aarch64, macOS x86_64 + aarch64, windows x86_64) and creates a GitHub Release.

## Conventions for AI assistants

- **Editing SKILL.md files**: preserve the YAML frontmatter exactly. Keep descriptions imperative ("Write …", "Design …") and under 200 chars. Keep tags lowercase and comma-separated.
- **Adding skills**: always use the template at `meta/skill-template/SKILL.md` as the starting structure. Include concrete BAD/GOOD code snippets, not just bullet points.
- **Never hand-edit `skills.json`**: run `python3 scripts/generate_manifest.py` instead. CI will block a PR where `skills.json` is out of sync with on-disk `SKILL.md` content.
- **Never strip zero-width characters** from markdown. They encode the DNA fingerprint.
- **Do not remove or re-order frontmatter fields** — `name` and `description` are required; `tags` is consumed by the CLI and manifest generator.
- **Path conventions matter**: category is derived from `parts[0]` and subcategory from `parts[1]` of the skill path. `languages/python/vanrossum/SKILL.md` → category=`languages`, subcategory=`python`. If you move a skill, the manifest reflects the new category automatically after regeneration.
- **Rust code (`cli/` and `tools/sk1llz-dna/`)** must pass `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`. Don't `#[allow]` clippy lints to make CI pass — fix the lint.
- **Python scripts** target Python 3.12 (see CI) and only depend on stdlib + `pyyaml` (for the validator). Don't introduce new dependencies without updating CI.
- **Don't create documentation files** (README, *.md outside skill directories) unless the task explicitly asks for it. Edit existing files instead.
- **High-signal skill content beats filler**. Quote primary sources. Include references.
