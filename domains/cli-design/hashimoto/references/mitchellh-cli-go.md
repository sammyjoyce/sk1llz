# `mitchellh/cli` Go library — field-tested traps

Load this file ONLY when implementing a Go CLI on top of the
`github.com/mitchellh/cli` library (the one powering Terraform, Vault,
Consul, Nomad, Packer). For other languages or framework choices, skip it.

The library is archived (2024-07-22) but remains the reference implementation
of Hashimoto's CLI patterns. Every quirk below exists because of a specific
real-world bug the HashiCorp team shipped into production.

## `RunResultHelp = -18511` — the help sentinel

`Command.Run(args []string) int` returns an exit code, but `-18511` is
reserved as a sentinel telling the framework "render my help text and exit
with that." Use this when a command is invoked with no args or with
`-h`/`--help` mid-parse — do NOT manually write help to stderr and return 1.

```go
func (c *FooCommand) Run(args []string) int {
    flags := c.Flags()
    if err := flags.Parse(args); err != nil {
        return cli.RunResultHelp  // NOT 1, NOT 2
    }
    // ...
}
```

Returning `cli.RunResultHelp` lets the framework write help to the
correct writer (see next section) and use the correct exit code globally.
Manual `fmt.Fprintln(os.Stderr, c.Help())` will bypass `HelpWriter`
customisation set by the embedding application.

## `HelpWriter` / `ErrorWriter` default to **stderr** for backwards compat

This is the single most counterintuitive quirk in the library. The library
comment literally says:

> `HelpWriter` is used to print help text and version when requested.
> Defaults to `os.Stderr` for backwards compatibility.
> **It is recommended that you set `HelpWriter` to `os.Stdout`, and
> `ErrorWriter` to `os.Stderr`.**

So the default is wrong and you must explicitly fix it in every new binary:

```go
c := cli.NewCLI("mytool", version)
c.HelpWriter = os.Stdout   // so `mytool --help | less` works
c.ErrorWriter = os.Stderr  // so errors don't pollute piped output
```

Why the confusion? Older HashiCorp binaries (pre-2016) printed help to stderr
because the POSIX tradition said "anything that isn't program output goes to
stderr, and help isn't program output." Modern consensus (and GNU conventions)
says explicit `--help` requests are user-requested output and belong on stdout
so that pagers work. The library preserved the old default to avoid breaking
existing users. In a new binary, override it.

## `CommandFactory` must be cheap — expensive init goes in `Run`

```go
c.Commands = map[string]cli.CommandFactory{
    "foo": func() (cli.Command, error) {
        return &FooCommand{}, nil  // cheap: allocate struct, return
    },
}
```

The factory is called **multiple times per execution** — at minimum once to
find the command, and again for `help` listings, autocomplete, and version
queries. If your factory opens a DB connection, reads config from disk, or
dials a remote service, every `mytool --help` will hit the network and
every tab-complete will open a file handle.

Defer to `Run()`:

```go
type FooCommand struct {
    client *api.Client  // nil until Run()
}

func (c *FooCommand) Run(args []string) int {
    if c.client == nil {
        c.client = buildClient()  // NOW it's OK to be expensive
    }
    // ...
}
```

## `BasicUi` is NOT thread-safe — wrap with `ConcurrentUi`

If your command spawns goroutines that write to the UI, a bare `BasicUi`
will interleave bytes mid-line, corrupting JSON output. The library ships
`ConcurrentUi` specifically for this:

```go
ui := &cli.ConcurrentUi{Ui: &cli.BasicUi{
    Reader:      os.Stdin,
    Writer:      os.Stdout,
    ErrorWriter: os.Stderr,
}}
```

Forgetting this manifests as rare, non-reproducible CI log corruption that
only appears under load. Always wrap for any multi-goroutine command.

## `HiddenCommands` and `FilteredHelpFunc` — two different tools

Both hide commands from users, but they behave very differently:

- **`HiddenCommands []string`** — the command still *runs* if invoked
  directly by name, but is omitted from `help` listings and autocomplete.
  Use this for `mytool internal-debug` or deprecated aliases you want to
  keep working but not advertise.

- **`FilteredHelpFunc(include, f)`** — wraps a help function to show only
  a whitelisted subset. The command still runs. Use this for multi-persona
  CLIs where `mytool --help` shows the user-facing surface, but
  `mytool --help --admin` shows everything.

Never confuse "hidden" with "removed." A hidden command with a dangerous
default is a loaded gun with the safety off. Use `HiddenCommands` for
deprecation, not secrecy.

## Autocomplete: install flag has no leading dash

```go
c.AutocompleteInstall = "autocomplete-install"  // NOT "-autocomplete-install"
```

The flag name is specified *without* dashes because the library accepts
both `-autocomplete-install` and `--autocomplete-install` at the call site.
If you include the dash in the option, you get `--` in user CLIs. Silent
bug — the install still works but looks ugly in help output.

## Longest-prefix subcommand matching with auto-created parents

Registering only `"foo bar"` as a nested subcommand causes the library to
auto-create `"foo"` as a no-op parent that just shows help for its children.
This is usually what you want, but has two sharp edges:

1. **`mytool foo qux`** — when only `foo bar` is registered, `qux` is
   silently treated as an argument to `foo`, not an error. The user sees
   the auto-generated `foo` help with no indication that `qux` was
   ignored. Override by returning `cli.RunResultHelp` from the auto-parent
   or register `foo` explicitly with a runnable that errors on unknown args.

2. **Help flag resolution** — `mytool foo bar -h` shows help for `foo bar`
   (correct), but `mytool foo -h` shows help for the auto-generated `foo`
   (minimal). Users expect `foo -h` to list subcommands. It does, but the
   output is sparse unless you customize `HelpFunc`.

## The 50-char `Synopsis()` constraint

```go
func (c *FooCommand) Synopsis() string {
    return "Create a new foo resource"  // <50 chars, imperative verb first
}
```

The library generates help listings by aligning command names in a column
followed by the synopsis. If you exceed ~50 characters, the listing wraps
ugly on an 80-column terminal. The library does not enforce this — it's
an honor-system constraint. Pair it with `Help() string` for the full
usage text (multiple paragraphs, flag descriptions, examples).

The discipline itself is the feature. If you cannot describe the command
in 50 characters starting with a verb, the command does too much — split it.
