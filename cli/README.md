# sk1llz CLI⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌​​‌‌​‍​​​​‌‌‌‌‍‌‌​‌‌​​​‍‌‌‌‌‌‌​​‍​​​​‌​‌​‍‌​‌​​​​‌⁠‍⁠

`sk1llz` is a small package manager for AI coding skills with an agent-first command contract.

The current CLI is intentionally narrow:
- read the catalog
- preview and apply installs/removals
- recommend skills from text or a repo path
- inspect the local environment
- expose machine-readable command metadata

## Installation

### From source

```bash
cd cli
cargo build --release
cp target/release/sk1llz ~/.local/bin/
```

### Shell completions

```bash
# Bash
sk1llz completions bash > ~/.bash_completion.d/sk1llz

# Zsh
sk1llz completions zsh > ~/.zfunc/_sk1llz

# Fish
sk1llz completions fish > ~/.config/fish/completions/sk1llz.fish
```

## Command tree

```text
sk1llz
  catalog list
  catalog search <query>
  catalog show <skill>
  catalog refresh
  install plan <skill>
  install apply <skill>
  remove plan <skill>
  remove apply <skill>
  recommend from-text [description]
  recommend from-path [path]
  env where
  env init
  env doctor
  describe [command...]
  completions <shell>
```

## Output contract

- `--format text|json` is available on every command.
- `--json` is a shortcut for `--format json`.
- Text is the default when stdout is a TTY.
- JSON is the default when stdout is piped.
- Primary output goes to stdout.
- Progress, prompts, and warnings go to stderr.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Local usage or runtime error |
| `3` | Requested item not found |
| `4` | Remote network or catalog error |

## Scope rules

- Repo-local installs live at `<repo-root>/.claude/skills/` when that directory already exists.
- Otherwise the default install scope is `~/.claude/skills/`.
- `sk1llz env init` creates `<repo-root>/.claude/skills/`.
- `--global` forces `~/.claude/skills/`.
- `--target` is allowed only as a relative path under the current directory.

## Agent-first surfaces

### 1. `describe`

Use `describe` to inspect the live command contract without scraping help text:

```bash
sk1llz describe
sk1llz describe install apply
sk1llz describe --json
```

### 2. Raw request bodies

Mutating install/remove commands accept either positional arguments or a raw JSON request:

```bash
sk1llz install plan hashimoto-cli-ux
sk1llz install apply --request '{"skill":"hashimoto-cli-ux","global":true}' --dry-run
printf '%s\n' '{"skill":"hashimoto-cli-ux"}' | sk1llz remove plan --request @-
```

### 3. Field masks

Read commands support `--fields` to trim response size:

```bash
sk1llz catalog list --limit 5 --fields id,name --json
sk1llz recommend from-text "rust cli ux" --fields score,reasons --json
```

## Examples

### Read the catalog

```bash
sk1llz catalog list
sk1llz catalog search distributed --json
sk1llz catalog show hashimoto-cli-ux
sk1llz catalog refresh --dry-run
```

### Preview and apply installs

```bash
sk1llz install plan hashimoto-cli-ux
sk1llz install apply hashimoto-cli-ux --yes
sk1llz install apply hashimoto-cli-ux --global --dry-run --json
```

### Preview and apply removals

```bash
sk1llz remove plan hashimoto-cli-ux
sk1llz remove apply hashimoto-cli-ux --yes
```

### Ask for recommendations

```bash
sk1llz recommend from-text "distributed systems in Go"
sk1llz recommend from-path .
```

### Inspect the environment

```bash
sk1llz env where
sk1llz env init --dry-run
sk1llz env doctor --json
```

## Development

```bash
# Format
cargo fmt

# Test
cargo test

# Try the schema surface
cargo run -- describe install apply

# Refresh the manifest cache
cargo run -- catalog refresh
```

## Notes

- The catalog cache lives at `~/.cache/sk1llz/skills.json`.
- The CLI now uses `reqwest` with `rustls`, so it builds without the system OpenSSL development package.
