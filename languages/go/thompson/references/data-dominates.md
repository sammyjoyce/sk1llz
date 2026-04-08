# Data Dominates: Tables over Control Flow⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​​‌​‍‌‌​‌‌‌‌‌‍​​​‌‌​‌​‍‌‌​‌​‌​‌‍​​​​‌​‌​‍​​‌‌‌​​​⁠‍⁠

Load this only when you are about to write (or refactor) a `switch`/`if` chain with **more than ~5 branches**, a parser, a state machine, or anything that looks like "interpret this input." Otherwise skip it.

> "Rule 5. Data dominates. If you've chosen the right data structures and organized things well, the algorithms will almost always be self-evident. Data structures, not algorithms, are central to programming." — Rob Pike

## Why this matters in Go specifically

Go's `switch` is cheap to write, which makes it easy to accumulate 30-arm monsters that are:
- impossible to diff review
- impossible to fuzz
- impossible to extend without touching the control flow
- impossible to generate from external specifications

The Thompson / Pike fix: **encode the decision as data**, then write a small, fixed interpreter over it. The data is testable, serialisable, and often generatable.

## The refactor, step by step

### Before: control-flow-driven HTTP status handler

```go
func describe(code int) string {
    switch {
    case code == 200:
        return "ok"
    case code == 201:
        return "created"
    case code == 204:
        return "no content"
    case code == 301:
        return "moved permanently"
    case code == 302:
        return "found"
    case code == 400:
        return "bad request"
    case code == 401:
        return "unauthorized"
    case code == 403:
        return "forbidden"
    case code == 404:
        return "not found"
    case code == 500:
        return "internal server error"
    // ... 40 more cases
    default:
        return "unknown"
    }
}
```

### After: the table *is* the program

```go
var statusText = map[int]string{
    200: "ok",
    201: "created",
    204: "no content",
    301: "moved permanently",
    302: "found",
    400: "bad request",
    401: "unauthorized",
    403: "forbidden",
    404: "not found",
    500: "internal server error",
}

func describe(code int) string {
    if s, ok := statusText[code]; ok {
        return s
    }
    return "unknown"
}
```

The table is diff-reviewable line-by-line. You can load it from a file. You can generate it from the IANA registry. You can test it as data (`for code, want := range statusText { ... }`). The interpreter is three lines that will never need to change.

## When the logic is more than a lookup: dispatch tables

For "run this function based on input," use a `map[K]func(...)` or a slice of structs with a function field. This is poor-man's object-orientation and is exactly what Pike recommends in *Notes on Programming in C*.

```go
type command struct {
    name string
    min  int // min args
    run  func(args []string) error
}

var commands = []command{
    {"add", 2, doAdd},
    {"rm",  1, doRemove},
    {"ls",  0, doList},
    {"mv",  2, doMove},
}

func dispatch(args []string) error {
    if len(args) == 0 {
        return errors.New("usage: tool <cmd> [args]")
    }
    for _, c := range commands {
        if c.name == args[0] {
            if len(args)-1 < c.min {
                return fmt.Errorf("%s: need %d args", c.name, c.min)
            }
            return c.run(args[1:])
        }
    }
    return fmt.Errorf("unknown command: %s", args[0])
}
```

Adding a command is **one line** in the table, not five in a switch. The linear scan is faster than a map for N < ~20 (fewer cache misses, no hash).

## State machines as transition tables

Classic anti-pattern: nested `switch` on `(state, event)`. Classic fix: a 2-D table.

```go
type state int

const (
    stateStart state = iota
    stateInWord
    stateInQuote
    stateEscape
    numStates
)

type event int

const (
    evLetter event = iota
    evSpace
    evQuote
    evBackslash
    evEOF
    numEvents
)

// transitions[state][event] = next state
var transitions = [numStates][numEvents]state{
    stateStart:   {stateInWord, stateStart, stateInQuote, stateInWord, stateStart},
    stateInWord:  {stateInWord, stateStart, stateInQuote, stateEscape, stateStart},
    stateInQuote: {stateInQuote, stateInQuote, stateStart, stateEscape, stateStart},
    stateEscape:  {stateInWord, stateInWord, stateInWord, stateInWord, stateStart},
}

func step(s state, e event) state {
    return transitions[s][e]
}
```

The transition table fits on one screen. You can diff it against a spec. You can visualize it with three lines of code. You can fuzz the interpreter trivially. Adding a state is one row; adding an event is one column. A spaghetti `switch` would be 40+ unreviewable lines for the same logic.

## The tell: when to table-ify

Refactor to a table the moment **any** of these is true:

1. You have more than 5 cases that all have the same shape (read one field, act).
2. You catch yourself writing comments like `// NOTE: keep in sync with X`.
3. The same `switch` appears in more than one file.
4. You want to generate the cases from external data (a spec, a registry, a schema).
5. You want non-programmers to add entries.

## When NOT to table-ify

Tables are wrong when:
- Each branch has genuinely different logic and arity (not "run a function"; actually different code paths with different locals).
- N ≤ 3. Two or three `if`s are clearer than a table.
- The branches have side effects the reader needs to see in reading order (e.g. a state teardown sequence).

The Thompson heuristic: if the `switch` body is *shaped the same* across arms, it is data in disguise. If each arm is a different shape, it is genuinely control flow. Trust the shape.
