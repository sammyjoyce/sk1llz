# CLI Conventions in the Thompson / Plan 9 Tradition⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌‌‌‌‌​‍‌‌​‌‌​‌‌‍​‌‌​‌‌​‌‍​​​‌​‌‌‌‍​​​​‌​‌​‍​‌‌‌​‌​‌⁠‍⁠

Load this when designing a **new** command-line tool or fixing a CLI's exit-code / flag / signal behaviour. Skip for library work or HTTP services.

These are the conventions Plan 9 and the Bell Labs Go authors (Pike, Cox, Griesemer) baked into Go's standard library. Scripts and pipelines written over the last 40 years depend on them. Breaking any one of them makes your tool a bad neighbour.

## Exit codes: the scripting contract

| Code  | Meaning                                    | Who sets it                                 |
|-------|--------------------------------------------|---------------------------------------------|
| `0`   | success                                    | normal return from `main`                   |
| `1`   | runtime error (couldn't do the job)        | explicit `os.Exit(1)` or `log.Fatal`        |
| `2`   | **usage error** (bad flags, missing args)  | `flag.Parse` does this for you              |
| `>=3` | program-specific conditions                | e.g. `grep` uses 1 for "no match", 2 for error |

**The `1` vs `2` distinction is load-bearing.** Shell scripts branch on it: `if ! mytool; then [[ $? -eq 2 ]] && show_help; fi`. Do not collapse them.

`grep`-style "signal through the exit code" is a legitimate Thompson trick: `grep pattern file` returns 0 if found, 1 if not found, 2 if error. This lets `grep -q pattern && do_thing` work without parsing output. Consider it for any "did I find it" tool.

## The `run() int` pattern (defer-safe exit)

`os.Exit` does not run deferred functions. This leaks files, temp dirs, and database connections. Always use this pattern:

```go
func main() {
    os.Exit(run())
}

func run() int {
    // All defers here WILL run before os.Exit.
    flag.Parse()

    f, err := os.Open(flag.Arg(0))
    if err != nil {
        fmt.Fprintln(os.Stderr, err)
        return 1
    }
    defer f.Close() // runs; os.Exit(run()) is outside this function

    if err := process(f); err != nil {
        fmt.Fprintln(os.Stderr, err)
        return 1
    }
    return 0
}
```

This is how `go` itself is structured (`cmd/go/main.go`). Copy it verbatim for every new tool.

## `flag` package conventions

The standard `flag` package is deliberately minimal. Resist `cobra`, `kingpin`, `urfave/cli` until you have a documented reason.

```go
var (
    n       = flag.Int("n", 10, "number of lines")
    verbose = flag.Bool("v", false, "verbose output")
)

func init() {
    flag.Usage = func() {
        fmt.Fprintf(os.Stderr, "usage: %s [-n N] [-v] [file...]\n", os.Args[0])
        flag.PrintDefaults()
    }
}

func main() {
    flag.Parse()
    // flag.Args() has positional args
}
```

Rules:
- **Single-letter flags for common options**, long names only when ambiguity would bite. `-n` not `--num-lines`.
- **No subcommand framework** unless you actually need subcommands. A 50-line tool with `cobra` is a code smell.
- **One flag per concept**, not `--enable-foo` and `--no-foo` both. Bool flags default to false; if you need the opposite, rename it.
- **Document the default in the usage string** — `flag.PrintDefaults()` does this automatically.

## Reading files: stdin is the default

```go
func main() {
    flag.Parse()
    args := flag.Args()
    if len(args) == 0 {
        if err := process(os.Stdin, "-"); err != nil {
            fmt.Fprintln(os.Stderr, err)
            os.Exit(1)
        }
        return
    }
    for _, name := range args {
        f, err := os.Open(name)
        if err != nil {
            fmt.Fprintln(os.Stderr, err)
            os.Exit(1)
        }
        err = process(f, name)
        f.Close()
        if err != nil {
            fmt.Fprintln(os.Stderr, err)
            os.Exit(1)
        }
    }
}
```

Note: no flag decides whether to read stdin or files. The presence of args decides. This is how `cat`, `grep`, `wc`, `sort`, and every other Unix filter have worked since 1971.

## Detecting a terminal (TTY)

Sometimes you want to print a progress bar only when stdout is interactive. Do not use a flag for this; detect it:

```go
import "golang.org/x/term"

isTTY := term.IsTerminal(int(os.Stdout.Fd()))
```

If `isTTY` is false, your tool is being piped; emit machine-readable output (no colours, no cursor moves, no spinners). This is why `ls` colourises in a terminal but not in `ls | less`.

## SIGPIPE: the polite death

When you run `mytool | head -1`, the kernel sends `SIGPIPE` to `mytool` as soon as `head` exits. In Go, the default behaviour is: the first write after the pipe closes returns an error with `syscall.EPIPE`, and subsequent writes keep failing. Ignoring this and looping prints error spam.

The Unix-correct behaviour is to **exit silently with status 0** on SIGPIPE. Pattern:

```go
if _, err := fmt.Println(line); err != nil {
    if errors.Is(err, syscall.EPIPE) {
        return 0 // downstream closed; we're done
    }
    return 1
}
```

Or install a signal handler that calls `os.Exit(0)` on `SIGPIPE`. Do not log "broken pipe" errors — they are not errors, they are the normal way pipelines terminate.

## Output discipline: data on stdout, diagnostics on stderr

- **stdout** is for the data your tool produces. Another program will probably read it.
- **stderr** is for human-readable diagnostics: errors, progress, warnings.
- Mixing these is the #1 way to break pipelines. `mytool file | grep foo` must not see the word "processing file..." in its input.

```go
fmt.Println(result)                              // stdout: data
fmt.Fprintln(os.Stderr, "warning: no encoding")  // stderr: diagnostic
```

Rule of silence (McIlroy): **when a program has nothing surprising to say, it should say nothing.** No "done." No "processing...". No "✨ success ✨". Silence on success is the Unix tradition because silence composes; "done." does not.

## Atomic file writes

Never overwrite a file in place. If the write fails midway, you destroyed the user's data.

```go
func writeAtomic(path string, data []byte) error {
    dir := filepath.Dir(path)
    f, err := os.CreateTemp(dir, ".tmp-*")
    if err != nil {
        return err
    }
    tmp := f.Name()
    defer os.Remove(tmp) // no-op if rename succeeded

    if _, err := f.Write(data); err != nil {
        f.Close()
        return err
    }
    if err := f.Close(); err != nil {
        return err
    }
    return os.Rename(tmp, path) // atomic on POSIX
}
```

Must be in the **same directory** as the target — `os.Rename` is not atomic across filesystems.

## Signal handling: graceful shutdown

For long-running tools (a one-off batch job, not a service):

```go
ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
defer stop()

if err := work(ctx); err != nil {
    if ctx.Err() != nil {
        return 130 // 128 + SIGINT, conventional
    }
    return 1
}
return 0
```

Exit code 130 is the shell convention for "killed by SIGINT." Honour it so `while mytool; do ...; done` in a shell loop terminates on Ctrl-C.

## Configuration: flags > env > file — in that order

If your tool needs configuration, evaluate in this precedence (highest wins):

1. Command-line flag
2. Environment variable (only for things that don't change per invocation — `$EDITOR`, `$HOME`, `$NO_COLOR`)
3. Config file (only if there are > ~5 settings)
4. Compiled-in default

If you only have 1–3 settings, **do not add a config file**. Flags are enough. A config file is a commitment to parse, validate, document, migrate, and handle errors from a new input format forever.

## The `NO_COLOR` and `TERM=dumb` conventions

- If `NO_COLOR` is set (to anything), disable ANSI colour. See [no-color.org](https://no-color.org).
- If `TERM=dumb` or unset and stdout is not a TTY, disable colour and cursor motion.
- Never require a flag to turn colour *off*; always detect and default to off in non-interactive contexts.

## The rule of composition, restated

A good Thompson-style tool:
- reads from stdin or files named on the command line
- writes data to stdout
- writes diagnostics to stderr
- exits 0/1/2
- is silent on success
- has no more flags than strictly necessary
- can be piped into and out of any other tool

If any of these are false, you are building an application, not a tool. That's fine — but it's a different style, and this skill does not cover it.
