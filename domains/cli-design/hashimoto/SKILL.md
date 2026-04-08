---
name: hashimoto-cli-ux
description: >-
  Design CLI tools using Mitchell Hashimoto's patterns from Vagrant, Terraform,
  Consul, Vault, Nomad, and Ghostty. Covers exit code semantics, stderr/stdout
  discipline, the plan-then-apply pattern, config precedence layering,
  TTY-aware output, and the doctor/path-help introspection commands. Use when
  designing CLI argument structure, help text, error messages, output formatting,
  subcommand hierarchies, machine-readable output, or interactive confirmation
  flows. Trigger keywords: CLI design, command-line UX, terminal tool, flags,
  subcommands, --help, --json, exit codes, error messages, progressive disclosure.
---

# Hashimoto CLI UX

Mitchell Hashimoto's CLIs (Terraform, Vault, Consul, Nomad, Ghostty) encode
hard-won decisions most CLI designers learn only through years of user pain.
This skill captures those decisions — not the basics Claude already knows.

## Thinking Framework

Before designing any CLI surface, ask yourself:

1. **Who calls this — a human or a script?** If both, you need dual output
   modes from day one. Retrofitting `--json` after users parse your text
   output with `awk` means you can never change the text format again.
2. **What breaks if this command is wrong?** Read-only commands get simple
   UX. State-mutating commands need the plan→confirm→apply pattern.
3. **What will the user try first?** Design for the obvious attempt.
   If someone types `vault` with no args, show a help summary — never
   a raw error or empty output.
4. **Can the user recover without Googling?** Every error must contain
   the fix. "Permission denied" is useless. "Permission denied. Run
   `vault login` or set VAULT_TOKEN" is Hashimoto-style.

## Expert-Only Design Decisions

### Exit Code Semantics (the Vault pattern)
Most CLIs use 0/1. Vault splits errors into two exit codes:
- **0** — success
- **1** — local/client error (bad flags, validation, wrong arg count)
- **2** — remote/server error (API failure, bad TLS, network timeout)

This distinction lets shell scripts differentiate "user typo" from
"infrastructure down" — critical for CI/CD pipelines. Apply this whenever
your CLI talks to a remote service.

### Stdout vs Stderr Discipline
HashiCorp CLIs enforce strict stream separation:
- **stdout** — only machine-parseable output (JSON, table data, tokens)
- **stderr** — progress, spinners, warnings, prompts, log messages

This lets `vault kv get -format=json secret/app | jq .data` work even
when the CLI prints warnings. NEVER mix human-readable decoration into
stdout. When you see interleaved output in CI logs, this rule was broken.

### The Plan→Apply Pattern
Terraform's most imitated design: separate "show what will happen" from
"do the thing." Apply this to any destructive or complex mutation:
```
mytool changes plan          # preview, exit 0 if no changes, exit 2 if changes
mytool changes apply         # execute, require --auto-approve or interactive confirm
mytool changes plan -out=f   # serialize the plan for deterministic apply
```
The key insight: `plan` returns exit code 2 when changes exist, 0 when
no-op. This lets CI conditionally run apply without parsing text output.

### Config Precedence Layering
HashiCorp tools use a strict 4-layer precedence (highest wins):
1. **CLI flags** (`-address=...`)
2. **Environment variables** (`VAULT_ADDR`)
3. **Config file** (`~/.vault`, project `.terraform.rc`)
4. **Compiled defaults**

Never invent a different order. Users internalize this once across all tools.
Always document which layer wins. Name env vars `TOOLNAME_FLAGNAME` —
Vault uses `VAULT_ADDR`, `VAULT_TOKEN`, `VAULT_FORMAT` consistently.

### TTY Detection Rules
Behavior MUST change based on whether stdin/stdout is a TTY:

| Condition | Behavior |
|---|---|
| stdout is TTY | Colors, tables, spinners, progress bars |
| stdout is pipe | No colors, no spinners, stable parseable format |
| stdin is TTY | Allow interactive prompts |
| stdin is pipe | Skip all prompts, require flags instead |
| `NO_COLOR` set | Disable all color regardless of TTY |
| `TERM=dumb` | Disable all color and cursor movement |

NEVER check only `--no-color`. You must also check `NO_COLOR` env var
(the cross-tool standard) and the TTY state. Forgetting TTY detection
is the #1 cause of garbled CI logs and broken pipe workflows.

### Subcommand Hierarchy Depth
HashiCorp CLIs use at most 2 levels: `vault secrets list`, `consul kv put`.
Pattern: `tool noun verb [args]`.

NEVER go deeper than 2 subcommand levels. `tool a b c d` is undiscoverable.
If you need more depth, your domain model is wrong — flatten it. Terraform
proved that even infrastructure management fits in 1 level (`terraform plan`,
not `terraform infrastructure plan create`).

## Anti-Patterns (with consequences)

**NEVER use `-v` for `--version`** in tools with a `--verbose` flag.
This collision is the most common flag naming mistake. Terraform uses
`-version` (no short form) and `-v` is undefined. Pick one meaning per
short flag and never reuse it.

**NEVER require interactive input without a non-interactive escape hatch.**
Every prompt must have a `--yes`/`-y` or `--auto-approve` flag. If stdin
is not a TTY and no flag was passed, print an error with the flag name —
never hang waiting for input that will never come. This breaks every CI
pipeline that runs your tool.

**NEVER print help to stdout.** Help text goes to stderr so that
`mytool --help | head` works but `mytool list > output.txt` doesn't
pollute the file with help text when the user forgets args. (Note: some
tools like `cobra`-based CLIs default to stdout — override this.)

**NEVER break backward compatibility in text output.** Once users parse your
output with grep/awk/sed, changing column order or formatting is a breaking
change. This is why `--json` must exist from v1 — it gives you a stable
contract while keeping text output free to evolve.

**NEVER silently succeed on destructive operations.** `terraform destroy`
requires typing "yes" interactively, or `--auto-approve` in scripts.
A bare `--force` flag with no confirmation has caused real production
outages. Make the dangerous path require deliberate effort.

## Hashimoto-Specific Patterns

### The `doctor` Command
Every tool should have a self-diagnostic command:
```
$ mytool doctor
  Checking config file...     OK
  Checking connectivity...    OK
  Checking auth token...      EXPIRED (3 days ago)
    Fix: Run 'mytool login' to refresh your token
  Checking version...         OUTDATED (v1.2, latest v1.5)
    Fix: Run 'brew upgrade mytool'
```
Rules: check everything that can go wrong, show OK/WARN/ERROR per check,
and always include a `Fix:` line for every failure. This eliminates 80%
of support tickets.

### The `path-help` / Introspection Pattern
Vault's `vault path-help sys/mounts` prints API documentation for any
endpoint directly from the CLI. If your tool wraps an API, expose this.
It turns the CLI into its own reference documentation.

### The `-output-curl-string` Debug Pattern
Vault's `-output-curl-string` flag shows the exact curl command that would
be equivalent to the CLI operation. Include this in any CLI that wraps
HTTP APIs — it makes debugging trust issues trivial because users can
reproduce the exact call outside your tool.

### Synopsis Under 50 Characters
`mitchellh/cli` enforces `Synopsis() string` on every command — a one-line
description under 50 characters shown in the help listing. This constraint
forces clarity. If you can't describe a command in 50 chars, the command
does too much.

## Decision Tree: Choosing Command Surface

```
Is the action read-only?
├─ Yes → Simple command, no confirmation needed
│        Use exit code 0 (success) or 1 (not found / error)
└─ No (mutates state)
   ├─ Is it reversible?
   │  ├─ Yes → Single command with -y/--yes flag
   │  └─ No (destructive)
   │     └─ Use plan→apply pattern
   │        Require explicit confirmation or --auto-approve
   └─ Does it talk to a remote service?
      ├─ Yes → Use exit code 1 (local) vs 2 (remote) split
      │        Add --output-curl-string for debugging
      └─ No → Standard 0/1 exit codes suffice
```

## Fallback Strategies

- **If your framework defaults help to stdout**: Override the help writer
  to stderr. In Go's `mitchellh/cli`, set `HelpWriter` to `os.Stdout` and
  `ErrorWriter` to `os.Stderr` (counterintuitive but documented).
- **If you can't implement plan→apply**: At minimum, add `--dry-run` that
  prints what would happen without doing it.
- **If autocomplete is too complex**: Ship a `completions` subcommand that
  generates shell scripts: `mytool completions bash > /etc/bash_completion.d/mytool`
- **If an error has no known fix**: Still include context: "Error: TLS
  handshake failed with api.example.com:443. Verify the server certificate
  or set VAULT_SKIP_VERIFY=1 (not recommended for production)."
