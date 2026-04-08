---
name: thompson-unix-philosophy
description: Write Go code in the Ken Thompson / Rob Pike Bell Labs tradition — data-dominates design, brute force before cleverness, text as interface, and programs that compose via stdin/stdout. Use when designing CLIs, filters, log processors, build tools, codegen, protocol parsers, or any Go program where simplicity, composition, and long-term maintainability matter more than feature breadth. Triggers: "unix philosophy", "filter program", "compose pipeline", "plan 9 style", "do one thing well", "brute force", "table-driven", "data-oriented Go", "pike style", "thompson style".
---

# Thompson / Pike Style for Go

> "When in doubt, use brute force." — Ken Thompson
> "Data dominates. If you've chosen the right data structures, the algorithms will almost always be self-evident." — Rob Pike, *Notes on Programming in C*, Rule 5

This is a **mindset skill**, not a cookbook. Load it when you must *decide* between two plausible Go designs and need a sharper razor than "clean code."

## Before you write any Go, ask these four questions

1. **What is the data?** Not the functions — the data. If you cannot draw the central data structure on a napkin, you do not yet understand the problem. Design the struct first; the functions fall out.
2. **How big is N, really?** If N is bounded by something a human edits (files in a repo, flags, config keys, CPU cores, lines in `/etc/hosts`), N ≤ ~10⁴ and a linear scan over a slice beats a `map` on wall-clock every time. Fancy structures have large constants and lose badly until N is genuinely large.
3. **Can this be a filter?** `stdin → transform → stdout` composes with every tool ever written. If yes, refuse to add flags that change *what* it reads or writes.
4. **What is the one reason this will change?** Not "what does it do" — "who files the bug that forces an edit." Two reasons to change = two programs. This is the real meaning of "do one thing."

If you cannot answer (1) in one sentence and (4) without listing alternatives, stop and redesign before writing code.

## Expert-only knowledge (the things practitioners learn the hard way)

### The data-dominates razor, stated precisely
Pike's complete list of data structures for "almost all practical programs" is: **array, linked list, hash table, binary tree**. Four. If your design needs a fifth (skip list, trie, Bloom filter, lock-free queue, B-tree), you owe yourself a benchmark proving the simpler choice loses on *your actual input*. In Go this means: reach for `[]T` first, `map[K]V` second, and treat `container/list`, `container/heap`, and third-party fancy structures as evidence of a design you haven't finished thinking about.

### Replace branches with tables
Long `if/else if` or `switch` chains are almost always a missing data structure. A state machine as a `[N][M]int` transition table is smaller, faster, testable as data, and diff-reviewable. This is the same insight behind Thompson's 1968 NFA regex construction: represent every state simultaneously as data, never as recursive control flow. See `references/data-dominates.md` before refactoring any `switch` with more than ~5 arms.

### "Brute force" is a state-space argument, not laziness
Thompson's 5- and 6-piece chess tablebases work because there are few *distinct positions* even when there are astronomically many *move sequences*. Before reaching for a "smart" algorithm, compute the actual state space. If it fits in RAM — even 10 GB of RAM today — the brute solution is correct by construction, has no edge cases, and compiles in a weekend. A clever solution is a bug generator you will babysit for years.

### Variable name length is inversely proportional to scope
Package-level: `maxPayloadBytes`. Function-local loop: `i`, `b`, `np`. A three-line function with `userAccountServicePointer` is *less* readable than one with `u`, because at that scope the context already tells you what it is. Long names in tiny scopes "jangle like bad typography" (Pike). The test: if a reader must scroll to understand a name, it is too short; if they must re-read the line to parse it, it is too long.

### Comments lie; the compiler does not
A comment is unchecked documentation that drifts the moment someone edits the code beside it. The only comments worth writing in Thompson-style Go are:
- A package-level doc on the **central data structure** explaining invariants (`// nodes[0] is always the root; parent < child indices`).
- A `// WHY` comment explaining a non-obvious *reason* (cache locality, protocol quirk, historical CVE).
- Never a `// WHAT` comment. `i++ // increment i` is the textbook Pike anti-example and it appears in production code constantly.

### Function names: verbs for actions, predicates for booleans
`if validSize(x)` beats `if checkSize(x)` because the reader instantly knows whether `true` means good or bad. In Go: name predicates `IsX`, `HasX`, `CanX`; name fallible actions after their *outcome* (`Parse`, `Open`, `Fetch`), not their activity (`DoParse`, `TryOpen`). Pike's rule: *procedure names reflect what they do; function names reflect what they return.*

### The Unix rule of silence, applied to Go errors
Error strings must compose when wrapped. Go's convention exists because of this:
- **lowercase**, no trailing punctuation, no `"failed to "` prefix
- `fmt.Errorf("parse %s: %w", path, err)` → `parse /etc/x: open /etc/x: no such file or directory`
- Capitalised or punctuated strings make wrapped errors read like ransom notes.

### Choose sentinel vs typed errors by *caller need*, not taste
- **Sentinel** (`var ErrNotFound = errors.New("not found")`) when callers branch on identity and carry no extra data — `io.EOF`, `sql.ErrNoRows`.
- **Typed** (`type *PathError struct`) when callers need structured fields via `errors.As`.
- **Plain `fmt.Errorf`** otherwise. If you are defining a sentinel "just in case someone wants to match it," delete it. Unused sentinels become API you cannot remove.

## Go-specific gotchas that enforce the philosophy

These are the silent failures that turn a Thompson-style program into a broken one. READ the referenced files before writing the relevant code.

| Situation | Gotcha | Fix |
|---|---|---|
| Reading lines > 64 KiB | `bufio.Scanner` **silently stops** with `bufio.ErrTooLong`; default `MaxScanTokenSize` is 65536. | `sc.Buffer(make([]byte, 0, 1<<20), 16<<20)` or use `bufio.Reader.ReadBytes('\n')`. |
| Exit from `main` | `os.Exit(n)` **does not run deferred functions** — open files leak, tempdirs stay. | `func run() int { defer cleanup(); ...; return code }; func main(){ os.Exit(run()) }` |
| Copying large streams | `io.Copy`'s buffer is hardcoded to **32 KiB**; changing it rarely helps. | If you *know* it matters, use `io.CopyBuffer` with a measured size. Otherwise do not parameterize. |
| Writing to a closed pipe | `fmt.Println` to stdout in `| head` returns `EPIPE`; ignoring it is the Unix-correct behavior. | Check `errors.Is(err, syscall.EPIPE)` and exit cleanly, or handle `SIGPIPE` via `signal.Notify`. |
| `flag` exit codes | `flag.Parse` calls `os.Exit(2)` on parse errors. **2 means misuse**. | Preserve this: usage errors = 2, runtime errors = 1, success = 0. Scripts depend on it. |
| `map` iteration order | Deliberately randomized per run. Tests that print a map will flake. | Sort keys before printing: `keys := maps.Keys(m); slices.Sort(keys)`. |
| `defer` in a loop | Defers stack until function return; holding 10⁶ files open will crash. | Extract the loop body into a function so `defer` fires each iteration. |

Before designing a CLI in this style, READ `references/cli-conventions.md`.
Before replacing a `switch`/`if` chain with a table, READ `references/data-dominates.md`.
Do NOT read those files for small edits or single-function changes — they are for design decisions.

## Anti-patterns (the wrong path, why it seduces, what it costs)

- **NEVER add a flag "just in case someone needs to configure it."** It seduces because it feels like humility ("I don't know what users want"). The cost: every flag is a permanent API surface, a documentation obligation, an interaction to test, and a future compatibility constraint. Thompson's rule: ship with zero flags; add one only after a real user hits a wall. **Instead:** ship the brute-force default and wait for the first real bug report.

- **NEVER write `io.Copy` variants to "avoid the 32 KiB hardcoded buffer."** It seduces because 32 KiB "feels small" on modern hardware. The cost: you lose the universal composability of `io.Copy` and gain nothing measurable — kernel readahead and page cache already dwarf user-space buffering for files; for network, latency dominates. **Instead:** benchmark first; use `io.CopyBuffer` only when the profile points there.

- **NEVER call `os.Exit(1)` inside a function that has `defer file.Close()` above it.** It seduces because it looks like clean error handling. The cost: the file handle leaks until the OS reclaims it, and on Windows the file stays locked. **Instead:** return an error all the way to `main`, or use the `run() int` pattern so `defer` still fires.

- **NEVER use `bufio.Scanner` on untrusted input without setting `Buffer`.** It seduces because `for scanner.Scan() { ... }` is the idiomatic loop. The cost: one 70 KB log line silently truncates your input, `Err()` returns `bufio.ErrTooLong`, and you process garbage. **Instead:** `sc.Buffer(make([]byte, 0, 64<<10), 16<<20)` or switch to `bufio.Reader.ReadBytes('\n')`.

- **NEVER define a type alias or wrapper struct for "clarity" that has no behavior.** `type UserID string` with no methods seduces because it looks type-safe. The cost: every call site needs conversions, every `fmt.Println` does the wrong thing by default, and you gained zero safety — `UserID("")` still compiles. **Instead:** keep the primitive, document the invariant, and only introduce a named type when you add a method.

- **NEVER capitalize an error string or end it with a period.** It seduces because it reads better in isolation. The cost: wrapped errors become `"Parse failed.: Open failed.: no such file"` — unreadable garbage. **Instead:** lowercase, no trailing punctuation, compose cleanly: `"parse %s: %w"`.

- **NEVER reach for `sync.Mutex` before you've tried a single goroutine owning the data and channels to talk to it.** It seduces because locking feels "how concurrent code is done." The cost: the bugs it introduces (deadlocks, races on fields you forgot, lock ordering) are exactly the bugs Go was designed to avoid. **Instead:** one goroutine, one owner, communicate via channels. Use a mutex only when profiling proves the channel hop is the bottleneck.

- **NEVER commit a regex without a benchmark if it runs on every line of user input.** It seduces because `regexp.MustCompile` is one line. The cost: Go's `regexp` (RE2) is linear-time but has a large constant — on simple prefixes, `strings.HasPrefix` is 20–100× faster. **Instead:** try `strings.Contains`/`HasPrefix`/`Cut` first; use `regexp` only when the pattern is genuinely irregular.

- **NEVER add a `// TODO` without a tracking issue and a date.** It seduces because it feels responsible. The cost: the file is now lying about its intent forever. **Instead:** either fix it now, or delete the thought and open an issue with the context.

## Decision tree: what to build

```
Is the input a stream of records?
├─ Yes → filter: stdin → transform → stdout. No flags that change I/O shape.
│        Use bufio.Scanner with explicit Buffer, or bufio.Reader for > 64 KiB lines.
└─ No → Is it a build/codegen tool?
        ├─ Yes → read files from args, write files atomically (tmp + rename).
        │        Exit 0 on success, 1 on failure, 2 on usage error.
        └─ No → Is it a long-running service?
                 ├─ Yes → NOT a Thompson program. Stop and pick a different style.
                 └─ No → One-shot utility: flags via `flag`, args via `flag.Args()`,
                          errors to stderr, data to stdout, exit code carries status.
```

## Fallbacks when brute force actually fails

Brute force fails in exactly three situations. Know them so you don't panic-optimize prematurely:

1. **State space exceeds RAM.** Measure first (`pprof --alloc_space`). If yes, stream with `bufio` and process in chunks; still prefer a linear scan per chunk.
2. **Real-time constraint (hard deadline < 1 ms).** Then `map` lookups lose to perfect hashing or sorted-slice binary search; profile before choosing.
3. **Quadratic on large N that is actually large.** If N > ~10⁵ and the algorithm is O(N²), you need a real algorithm. Not before.

Everything else — "it feels slow," "it might scale," "users might have big inputs" — is Pike's Rule 1: *you cannot tell where a program will spend its time.* Measure, then fix only the part that overwhelms the rest.
