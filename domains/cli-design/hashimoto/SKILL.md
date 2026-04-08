---
name: hashimoto-cli-ux
description: >-
  Design operator-grade CLIs in the Hashimoto style: human text is disposable,
  machine output is a versioned contract, mutation workflows separate review
  from execution, and automation never depends on terminal heuristics. Use when
  designing or refactoring subcommands, help/version behavior, --json or event
  streams, plan/apply or dry-run flows, flag-env-config precedence, config
  introspection, CI-safe prompts, or full-screen terminal UX. Trigger keywords:
  CLI design, command-line UX, exit codes, automation, machine-readable output,
  plan apply, dry-run, stderr stdout, config precedence, help text, subcommands,
  TTY, terminal flicker.
---

# Hashimoto CLI UX⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌‌​‌‌​​‍‌​​​​​‌‌‍‌​​‌​‌​​‍‌​‌​​‌‌​‍​​​​‌​‌​‍​‌​​​​​‌⁠‍⁠

Hashimoto-style CLIs optimize for operators who alternate between three modes:
interactive use, automation, and recovery under stress. The design target is
not "pleasant flags." It is a command surface that stays trustworthy after
users pipe it through wrappers, code review, CI, and incident response.

## Load The Right Depth

- MANDATORY: Before implementing a Go CLI on `github.com/mitchellh/cli` or `github.com/hashicorp/cli`, read [`references/mitchellh-cli-go.md`](references/mitchellh-cli-go.md).
- MANDATORY: Before designing CI behavior, wrappers, or exit-code semantics, read [`references/ci-automation-traps.md`](references/ci-automation-traps.md).
- Do NOT load either reference for ordinary naming, help-copy, or subcommand grouping work. This file is enough for general CLI surface design.

## Core Contract

1. Human text is not an integration surface. If a command will be consumed by
   automation, give it a versioned machine contract from day one. Terraform's
   machine-readable UI always starts with a `version` message and requires
   consumers to ignore unknown fields in minor versions but reject unknown major
   versions. Copy that rule, not just the `--json` flag.
2. Pick the machine shape by execution shape. For bounded queries, emit one
   JSON object with a `format_version`. For long-running or concurrent work,
   emit NDJSON events, one object per line, with timestamps, stable message
   types, and resource identities. If consumers need to correlate interleaved
   work, plain text logs are already a broken design.
3. `--json` means "non-interactive contract," not "same command plus prettier
   output." Terraform makes `-json` imply `-input=false`. If machine mode can
   still prompt, wrappers deadlock and the contract is false advertising.
4. Exit codes are for control flow, not taxonomy. Default to `0=success,
   1=failure`. Add a third code only for "successful but actionable difference"
   such as Terraform's `-detailed-exitcode` plan result. Do not burn extra exit
   codes on every error class unless shells genuinely branch on them.
5. Separate speculative review from executable intent. Terraform's unsaved plan
   is for review; the saved plan file is an opaque execution artifact and can
   contain cleartext sensitive data. If approval matters, the review text and
   the apply artifact cannot be the same thing.
6. Make precedence boring: explicit flags > env-injected defaults > config file
   > compiled default. If you support env-injected flags, insert them after the
   subcommand so explicit CLI flags still win, as `TF_CLI_ARGS[_name]` does.
7. Treat workflow state as first-class. Any env var that changes workspace,
   target host, or data directory must remain stable from init through apply or
   later commands will fail in ways that look unrelated. Terraform documents
   this explicitly for `TF_DATA_DIR`, and warns that `TF_WORKSPACE` is safest in
   non-interactive automation because humans forget it is set.
8. Shells are lossy. When values are structured, typed, or secret, support
   `@file` and stdin `-` input, not just `key=value` flags. Vault also exposes
   `-field` output for a single value without the usual table wrapper or
   trailing newline, which avoids forcing scripts to parse a table or trim
   output before command substitution.
9. Self-document the live system. `vault path-help`, `ghostty +show-config
   --default --docs`, and `-output-curl-string` all reduce support load because
   the running binary can explain its own routes, effective config, or wire
   request without sending the user to a website first.
10. Diagnostics need calibrated severity, not vibes. Vault's `operator
    diagnose` uses `[success]`, `[warning]`, `[failure]`, bubbles child
    severity upward, and publishes actual thresholds such as warn over 100ms and
    fail over 30s for storage access. "This might be slow" is not operator UX.
11. Full-screen CLIs do not get to ignore terminal performance anymore.
    Ghostty's docs call out screen tearing and recommend Synchronized Output
    plus partial redraws instead of clearing whole rows or the entire screen.

## Before You Choose A Surface, Ask Yourself

- Is this output a human convenience or a stable API? If the answer is "both,"
  design the machine contract first and let the prose ride on top of it.
- Is the user approving a proposal or invoking an artifact? Proposals can be
  re-run; executable artifacts must be replayable, opaque, and treated as
  sensitive.
- Which hidden state must remain constant between commands? Directory, data
  dir, workspace, auth context, plugin/provider versions, and included config
  files are workflow state, not trivia.
- Where will fidelity be lost first: shell quoting, wrapper scripts, logs,
  environment propagation, or terminal redraw loops? That failure point should
  shape the interface.
- If this fails in CI at 2 a.m., does the error text name the exact flag, env
  var, or introspection command that fixes the class of failure?

## Freedom Calibration

- High freedom: command naming, noun/verb grouping, synopsis wording, examples,
  and how much next-step coaching you show to humans.
- Low freedom: stdout vs stderr contract, prompt behavior in machine mode,
  exit-code semantics, schema versioning, and secret-handling paths.
- Medium freedom: confirmation flows, config layering, and introspection
  commands. These vary by product, but the invariants above do not.

## Decision Tree

- Need machine consumption?
  - One-shot result: single JSON object with `format_version`.
  - Long-running or concurrent work: NDJSON events; emit schema version first.
- Need mutation?
  - Low-blast-radius change: one command plus `--dry-run` and `-y`.
  - Review-required or irreversible change: separate preview from executable
    artifact; document how drift invalidates earlier previews.
- Need user-supplied structured data or secrets?
  - Small scalar safe to echo: flag.
  - Complex, typed, or secret: `@file`, stdin `-`, or config input.
- Need persistent configuration?
  - One-off override: flag.
  - Automation default: env var or injected args.
  - Durable operator preference: config file.
  - Startup-only behavior: CLI-only option, even if most settings live in
    config. Ghostty explicitly does this for `config-default-files`.
- Need nested subcommands?
  - If success depends on autogenerated parents or longest-prefix matching,
    flatten the tree or register the parent explicitly so unknown subcommands do
    not get misread as positional args.

## Anti-Patterns You Only Learn The Hard Way

- NEVER promise `--json` and still allow prompts because it feels flexible. In
  automation it deadlocks wrappers and violates parse contracts. Instead make
  machine mode imply non-interactive execution and fail fast on missing inputs.
- NEVER treat human-readable preview text as the executable approval artifact
  because transparency is seductive. The real system can drift, and saved plans
  or debug bundles may carry cleartext secrets. Instead separate speculative
  review from an opaque apply artifact and treat that artifact like a credential.
- NEVER let wrappers normalize or hide exit codes because captured stdout/stderr
  seems helpful. CI loses the only cheap branch signal it had. Instead preserve
  raw process status end to end and document shell pitfalls when users pipe
  through `tee`, pagers, or action wrappers.
- NEVER force structured or secret values through shell-quoted flags because it
  looks simple. You lose typing, quoting, and confidentiality in one move.
  Instead accept `@file`, stdin `-`, or full JSON on stdin.
- NEVER rely on TTY detection alone to distinguish humans from automation
  because many CI systems allocate PTYs and some local users pipe explicit help
  into pagers. Instead use an explicit automation knob and treat TTY only as a
  rendering hint.
- NEVER let autogenerated parent commands define your information architecture
  because nesting looks free. Users get sparse help, and unknown subcommands can
  be swallowed as positional args. Instead register parents deliberately and
  keep synopsis lines under about 50 characters so listings stay scannable.
- NEVER pretend config inclusion order is obvious because "it follows file
  order" is almost always wrong once nesting exists. Ghostty loads included
  files after the containing file and warns on cycles. Document your rule
  precisely, support optional includes if useful, and make CLI-only settings
  explicit rather than silently half-working in config files.
- NEVER redraw the whole screen every frame because it seemed fine on a slower
  terminal. Fast renderers expose tearing immediately. Instead implement
  Synchronized Output and update only the cells that changed.

## Fallbacks When The Ideal Design Is Not Ready Yet

- Cannot ship a full review/apply split yet: provide `--dry-run` plus a saved
  request artifact, but never pretend the human preview is replayable.
- Cannot ship a rich event stream yet: emit one versioned JSON result at the
  end and keep progress or spinners on stderr.
- Wrappers or CI glue keep swallowing exit codes: disable the wrapper, or add a
  log/artifact file so users do not need to pipe through `tee` just to persist
  output.
- Terminal UI still flickers: reduce redraw scope first, then add a plain mode
  for `TERM=dumb` or similar degraded terminals before shipping more animation.

## Practical Defaults

- `tool --help` and `tool --version` are explicit user-requested output; send
  them to stdout. Usage or parse failures go to stderr. If you use
  `mitchellh/cli`, override the legacy default so `HelpWriter` is stdout.
- If you expose a "helpful human" automation knob like `TF_IN_AUTOMATION`,
  keep it cosmetic. The exact prose can change between minor versions; the
  machine contract cannot.
- Publish at least one introspection path for API-backed tools: route help,
  effective config, raw request preview, or a focused diagnostic bundle.
- If you expose diagnostics, say when they are meaningful and when they are only
  advisory. Vault diagnose is safe to run when the server is down, but some
  checks become meaningless if the server is already running.
