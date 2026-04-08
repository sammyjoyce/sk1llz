# Go Concurrency: Expert Gotchas⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌​‌​‌​‍​‌‌​‌‌‌‌‍​‌‌‌‌‌​‌‍​‌‌‌​‌​​‍​​​​‌​‌​‍​​‌​​​‌​⁠‍⁠

Load this file when: reviewing Go code with goroutines and channels, debugging hangs/leaks/data races, or architecting a concurrent Go service.

## Rob Pike's own admission (the one to start with)

From his 14-year Go retrospective (2023): *"Countless programmers tried to make their code faster by parallelizing it using goroutines, and were often baffled by the resulting slowdown. Concurrent code only goes faster when parallelized if the underlying problem is intrinsically parallel, like serving HTTP requests. We did a terrible job explaining that."*

**Concurrency is structure. Parallelism is execution.** Goroutines give you the structure. Parallelism requires that the work was actually independent *and* that you had unused cores. Neither is guaranteed.

## Channels vs. mutexes — the rule the docs don't state clearly

The standard Go koan is "don't communicate by sharing memory; share memory by communicating." Taken literally, this has made thousands of programs slower and more complex. The correct rule, stated by the Go team in `sync` package docs and several Russ Cox talks:

**Channels transfer *ownership* of data. Mutexes protect *shared state*.**

| If you have…                                      | Use…            |
| ------------------------------------------------- | --------------- |
| A counter, cache, or map many goroutines touch   | `sync.Mutex` or `sync.RWMutex` |
| A work item produced here, consumed there        | channel         |
| A cancellation signal                             | `context.Context` or `<-chan struct{}` |
| A "wait for all workers to finish"                | `sync.WaitGroup` |
| A one-shot initialization                         | `sync.Once`     |
| A critical section < 1 µs                         | `sync.Mutex` (channels have ~50–100 ns overhead per op) |
| A counter in a tight loop                         | `sync/atomic`   |

Channel-as-mutex is **≈10× slower** than `sync.Mutex` for pure critical-section work, because every send/receive goes through the scheduler.

## The goroutine leak pattern

Every `go f()` must have an answer to: **"When does this stop?"** If the answer is "when the program exits," you are writing a leak. Detection: `runtime.NumGoroutine()` growing without bound, or `pprof` goroutine profile showing thousands of workers blocked on the same line.

### The canonical leak

```go
// LEAK: if nobody reads from ch, the goroutine blocks on send forever.
func fetch(url string) <-chan string {
    ch := make(chan string)
    go func() {
        ch <- slowHTTP(url)  // blocks here if caller disappears
    }()
    return ch
}
```

### The fix pattern

```go
func fetch(ctx context.Context, url string) <-chan string {
    ch := make(chan string, 1) // buffered so send never blocks
    go func() {
        result := slowHTTP(url)
        select {
        case ch <- result:
        case <-ctx.Done():
        }
    }()
    return ch
}
```

**Two fixes in one**: the buffered channel (capacity 1) lets the producer finish even if no one reads; the `select` lets the producer exit on cancellation even if the buffer is full.

## `GOMAXPROCS` and the CPU-bound pool size

- **I/O-bound work** (HTTP clients, DB queries, file reads): goroutines are cheap because blocked ones cost only their stack (~8 KB initially, resizable). Thousands are fine.
- **CPU-bound work**: creating more goroutines than `runtime.GOMAXPROCS(0)` for pure CPU work makes things **slower**, because the scheduler thrashes. The rule is: `workers = GOMAXPROCS` for CPU-bound, bounded work queue.
- **Mixed**: use a bounded worker pool with `GOMAXPROCS` workers and a queue; do not spawn one goroutine per task.

**Container gotcha:** `GOMAXPROCS` defaults to `runtime.NumCPU()`, which reads the host CPU count, *not* the cgroup quota. A Go service in a Kubernetes pod with a CPU limit of 2 cores on a 96-core node will default to `GOMAXPROCS=96` and scheduler-thrash itself. Use `uber-go/automaxprocs` or set `GOMAXPROCS` explicitly from the cgroup limit. This is a common production-slowdown cause and is not in any default Go tutorial.

## The `nil error` interface pitfall

```go
type MyError struct{ msg string }
func (e *MyError) Error() string { return e.msg }

func doWork() error {
    var err *MyError  // nil pointer
    // ... nothing went wrong, err stays nil
    return err        // returns a non-nil interface wrapping a nil pointer!
}

if doWork() != nil {
    fmt.Println("error!") // prints even though nothing went wrong
}
```

The interface value `(type=*MyError, value=nil)` is **not** equal to `nil`. A `nil` interface means both type *and* value are nil. The fix: return `nil` of the interface type explicitly.

```go
func doWork() error {
    var err *MyError
    if somethingBad() {
        err = &MyError{...}
        return err
    }
    return nil  // explicit interface nil
}
```

This is in the Go FAQ, but every team hits it at least once. The only reliable fix is to always return the interface type (`error`), never a concrete pointer type, from anything that can be compared to `nil`.

## `sync.Mutex` is not re-entrant

A Go mutex cannot be locked twice by the same goroutine. Doing so deadlocks permanently. If you're tempted to use a re-entrant lock, your design has a function calling back into itself through a lock boundary — restructure so the locked section is the innermost scope.

## Data races are not UB — they're worse

Go's memory model (updated 2022) says that a data race is a "program error." In practice, the runtime will sometimes *crash* with a garbage heap pointer, but more often it will silently read a torn 64-bit value on 32-bit ARM, or observe a half-written map header and segfault inside the runtime. There is no "benign race" in Go. Always run tests with `go test -race` on CI; it catches most races with ~5× slowdown.

`-race` does **not** catch races where neither goroutine actually runs concurrently during the test (e.g., races gated by a timer or a rare code path). Fuzz tests + `-race` catch more.

## Channels and `select` ordering

`select` chooses uniformly at random among ready cases. It is **not** first-come, not declaration order. If you write a worker that prefers "work" over "cancel," you cannot express that with a single `select`:

```go
// BUG: if both ready, 50/50 whether you cancel or do work
select {
case w := <-workCh:
    handle(w)
case <-ctx.Done():
    return
}
```

The idiom for "drain work before cancel" is a nested select:

```go
for {
    // Priority check: cancellation wins if pending.
    select {
    case <-ctx.Done():
        return
    default:
    }
    // Normal wait.
    select {
    case w := <-workCh:
        handle(w)
    case <-ctx.Done():
        return
    }
}
```

## References

- Pike, R. (2023). *What We Got Right, What We Got Wrong*. https://commandcenter.blogspot.com/2024/01/what-we-got-right-what-we-got-wrong.html
- Pike, R. (2012). *Concurrency Is Not Parallelism*. https://www.youtube.com/watch?v=oV9rvDllKEg
- Go Memory Model (2022). https://go.dev/ref/mem
- Cox, R. Channels-vs-mutexes guidance in `sync` docs. https://pkg.go.dev/sync
- uber-go/automaxprocs. https://github.com/uber-go/automaxprocs
