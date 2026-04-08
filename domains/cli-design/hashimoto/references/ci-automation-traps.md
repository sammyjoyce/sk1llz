# CI / automation traps in Hashimoto-style CLIs

Load this file ONLY when diagnosing or designing CI integration for a
Hashimoto-style CLI, or when a user reports "the exit codes are wrong in
my pipeline." These are the non-obvious failure modes.

## `terraform plan` exit code is `0` by default — even when changes exist

The most-cited fact about Terraform's plan-apply design is subtly wrong:

```
$ terraform plan     # changes waiting? still exits 0
$ echo $?
0
```

The three-state exit code (0=no-op, 1=error, 2=changes) is gated behind
the `-detailed-exitcode` flag:

```
$ terraform plan -detailed-exitcode
$ echo $?
2
```

Why the opt-in? Because `set -e` pipelines break when a "successful plan
with changes" exits non-zero. HashiCorp chose backward compatibility with
naive bash scripts over discoverability. The cost is that every serious
Terraform CI tutorial must spend a paragraph explaining this flag.

**Design lesson for your own CLI:** if you adopt the three-state pattern,
make it the default and accept the `set -e` breakage. A silent default
that hides critical state is worse than a loud one that forces users to
write `set +e` for one command. Document both approaches and pick one
deliberately.

## The `${PIPESTATUS[0]}` trap

```bash
terraform plan -detailed-exitcode | tee plan.txt
echo $?   # always 0 (the tee succeeded)
echo ${PIPESTATUS[0]}   # the actual plan exit code
```

Any pipe through `tee`, `grep`, `less`, or `head` destroys `$?`. Bash
stores the original in `PIPESTATUS`, zsh in `pipestatus`, fish in
`$pipestatus`, POSIX sh in nothing (use `set -o pipefail` instead).

If your CLI docs show piped examples, add a note. Better: ship a
`--log-file=FILE` flag so users don't need to pipe at all. Terraform
has `TF_LOG_PATH`; Vault has `-log-file`.

## `TF_IN_AUTOMATION` — the environment variable nobody knows about

Terraform checks `TF_IN_AUTOMATION` (any non-empty value) and silently
adjusts its output:

- Suppresses the "Did you mean to use -out?" hint after `plan`
- Removes the "Run `terraform apply` to apply these changes" footer
- Omits interactive upgrade suggestions

The variable is documented exactly once in the Terraform manual under
"Running Terraform in automation." CI vendors almost never set it, so
most pipelines have noisy output full of hints aimed at humans.

**Design lesson:** provide an escape valve for your "helpful human
guidance" output. Name it `TOOL_IN_AUTOMATION` or `CI=true` (respect the
widely-set convention). Do not rely on TTY detection alone — many CI
systems allocate PTYs to capture output properly, so `isatty()` returns
true in CI.

## The GitHub Action wrapper that swallows exit codes

`hashicorp/setup-terraform@v3` installs a wrapper around `terraform` that
captures stdout/stderr and exposes them as step outputs. The wrapper
normalizes exit codes, so `terraform plan -detailed-exitcode` returns 0
even when there are changes. The fix is buried in the action's README:

```yaml
- uses: hashicorp/setup-terraform@v3
  with:
    terraform_wrapper: false   # give me the raw exit codes
```

**Design lesson:** if you ship both a binary AND an ecosystem wrapper,
the wrapper must either preserve exit codes transparently or document
the deviation in a hazard-level warning, not a footnote.

## SIGHUP to hot-reload log level (Vault pattern)

Vault servers honor `SIGHUP` to re-read the log level from config. This
overrides even values set via CLI flag or environment variable — the
signal is authoritative for the running process:

```
$ kill -HUP $(pidof vault)
# vault re-reads log_level from config file and applies it
```

This lets operators debug a running production server without restarting
it. The precedence inversion (signal > env > flag) is unusual and
deliberate: you are shouting at a running process, so your shout wins.

**Design lesson:** long-running daemons should expose at least one
runtime-tunable knob via signal. Log level is the minimum. Some HashiCorp
tools also re-read TLS certs on SIGHUP so you can rotate without downtime.

## `VAULT_SKIP_VERIFY` — the escape valve that destroys security

Every Hashimoto CLI has a flag or env var for skipping TLS verification:

- `VAULT_SKIP_VERIFY=1`
- `CONSUL_HTTP_SSL_VERIFY=false`
- `terraform ... -insecure` (for HTTP backends)

Users *will* set these and forget to unset them. The pattern HashiCorp
evolved toward: when TLS verification is off, print a red warning to
stderr on every single invocation — not just the first one. Annoyance is
the feature. A silent insecure mode will end up in a production script.

```
WARNING: TLS verification is disabled (VAULT_SKIP_VERIFY).
         This is not safe for production use.
```

Never suppress this warning with a flag. If users complain it clutters
their logs, point them at fixing their certificate chain instead.

## Log level reload via env var for forked children

`TF_LOG=TRACE terraform apply` enables trace logging for the parent
process AND all provider subprocesses Terraform forks. Because providers
are separate binaries, Terraform propagates `TF_LOG` through the
environment rather than via flags.

**Design lesson:** if your CLI spawns subprocesses (plugins, providers,
hooks), diagnostic controls must flow through the environment, not
command-line flags. Flags don't cross process boundaries; env vars do.
Pick an env var name early and commit to it — renaming `TF_LOG` is now
effectively impossible because every CI system has it baked in.
