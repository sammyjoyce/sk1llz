# Decomposing an Inheritance Hierarchy into Orthogonal Policies⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​​‌​‍‌‌‌‌​​‌‌‍​‌‌‌​‌‌‌‍‌‌‌​​​‌‌‍​​​​‌​‌​‍‌​‌‌​‌‌‌⁠‍⁠

Load this file only when you are **actually refactoring an existing class** into policies, or **designing one from scratch** and have already answered "yes" to the five questions in `SKILL.md`. For deciding whether to use policies at all, the decision tree in `SKILL.md` is sufficient — don't load this.

## The procedure (seven steps)

### Step 1 — Enumerate the axes of variation

List every behavioral axis the class already exhibits or will exhibit. Write it as a table:

| Axis | Values observed today | Values conceivable in 6 months |
| --- | --- | --- |
| Threading model | single-threaded, mutex-locked | lock-free |
| Error handling | throw, return optional | error code, `std::expected` |
| Allocation source | `new`/`delete`, stack buffer | arena, pool |

**Rule:** if a column has only one value today AND no concrete second implementation is on the team's 6-month roadmap, that axis is **not** a policy. It is a hardcoded choice you can revisit later.

### Step 2 — Check orthogonality pairwise

For every pair of axes `(A, B)`, ask: "does the correct choice on A constrain the correct choice on B?" If yes, collapse them into one policy or accept that you have one axis, not two.

Worked example: `(Threading, Allocation)` is orthogonal — a lock-free container is legal with any allocator. `(Threading, IteratorInvalidation)` is **not** orthogonal — lock-free containers typically have stronger iterator invalidation rules than locked ones. That is one axis pretending to be two.

### Step 3 — Count. If >3, stop and restructure

If you end up with four or more orthogonal policies, one of the following is true:

- You have discovered two classes pretending to be one. Split them.
- One axis is actually a trait derived from another. Compute it.
- One axis is configuration, not variation. Hardcode the common case and ship the other as a named sibling class.

The three-policy ceiling is empirical, not theoretical. Every real-world Loki-style class that crossed it (including several in Loki itself) became known as "the one with too many template parameters" and grew a type-erased facade later.

### Step 4 — Name the contract, not the implementation

Bad: `MutexPolicy`, `NewDeletePolicy`, `ThrowOnErrorPolicy`.
Good: `ThreadingModel`, `AllocationStrategy`, `ErrorReporting`.

The concept outlives any particular implementation. Users instantiate with `LockFreeThreadingModel`, not with `LockFreePolicy`. Reading the code should tell you what axis is being varied, not which implementation the author grabbed first.

### Step 5 — Write the concept before the first policy

With C++20 concepts, write the constraint first:

```cpp
template <class M>
concept ThreadingModel = requires {
    typename M::mutex_type;
    typename M::lock_type;
    requires std::default_initializable<typename M::mutex_type>;
};
```

Now implement the cheapest possible policy that satisfies it (the stateless, single-threaded one), instantiate the host, and ship. Add heavier policies only when a caller demands one.

**Why first-draft the concept:** it prevents the policy from leaking implementation details. If the concept requires a member you don't need, you've over-specified and locked users out of future implementations. The concept is the contract; everything else is negotiable.

### Step 6 — Store stateless policies with `[[no_unique_address]]`, never inherit

See `modern-replacements.md §3` for the exact code and the MSVC spelling. Do not use private inheritance for EBCO in new code. The consequences of the `final`-policy footgun and member-name collisions outweigh the 3 lines you save.

### Step 7 — Add size assertions for every stateless policy

```cpp
static_assert(sizeof(Container<int, SingleThreaded, ThrowOnError>)
              == sizeof(Container<int, LockFree, ReturnExpected>),
              "stateless policies must not add storage");
```

These catch the moment someone makes a policy stateful by accident, or adds a member that defeats `[[no_unique_address]]`. The build fails at the assertion line instead of at the profiler six months later.

## Worked example — inheritance to policies

### Before (the sin)

```cpp
class MessageQueue : public LockingBase, public LoggingBase, public ErrorThrowing {
public:
    void enqueue(Message m) {
        LockingBase::acquire();
        LoggingBase::log("enqueue");
        if (buffer_.full()) ErrorThrowing::fail("full");
        buffer_.push(std::move(m));
        LockingBase::release();
    }
private:
    RingBuffer<Message, 1024> buffer_;
};
```

Problems:
- Three orthogonal concerns baked in by inheritance.
- To ship a single-threaded, no-logging, noexcept variant you must fork the class.
- `LockingBase::fail` and `ErrorThrowing::fail` collide silently if both ever define it.
- `LoggingBase` cannot be `final` without defeating EBCO.

### After (the fix)

```cpp
template <class M> concept ThreadingModel = /* ... */;
template <class L> concept Logger         = /* ... */;
template <class E> concept ErrorReporting = /* ... */;

template <
    ThreadingModel Threading = SingleThreaded,
    Logger         Logging   = NullLogger,
    ErrorReporting Errors    = ThrowOnError>
class MessageQueue {
public:
    void enqueue(Message m) {
        typename Threading::lock_type _(mtx_);
        log_.record("enqueue");
        if (buffer_.full()) {
            return errors_.report("full");
        }
        buffer_.push(std::move(m));
    }
private:
    POLICY_MEMBER typename Threading::mutex_type mtx_;
    POLICY_MEMBER Logging                        log_;
    POLICY_MEMBER Errors                         errors_;
    RingBuffer<Message, 1024>                    buffer_;
};

static_assert(sizeof(MessageQueue<>) == sizeof(RingBuffer<Message, 1024>),
              "stateless default policies must not add storage");
```

### What we gained
- Three orthogonal axes, each hardcoded today to zero-cost defaults.
- `static_assert` guarantees the defaults stay zero-cost.
- Concepts produce readable errors when users pass nonsense.
- Users who need the heavy version instantiate with `MessageQueue<MutexThreading, StderrLogger, ReturnExpected>`.

### What we did NOT do
- We did **not** expose `MessageQueue` as a vocabulary type crossing API boundaries. If it flows into public headers that multiple teams share, we type-erase it behind `IMessageQueue` or provide named aliases `SyncMessageQueue` / `AsyncMessageQueue`.
- We did **not** give `MessageQueue` a virtual destructor. It is a value type. If users want polymorphism they hold it inside a type-erased wrapper.
- We did **not** inherit from the policies. `[[no_unique_address]]` members handle EBCO without the `final`-policy trap.

## Red flags during decomposition

| Red flag | What it means | What to do |
| --- | --- | --- |
| One policy's interface references another policy's type | Axes are not orthogonal | Fuse them or rethink |
| A policy has both `allocate` and `log` | Policy does two things | Split |
| A policy's only implementation today is empty | You are over-templatizing | Hardcode and delete the parameter |
| A policy has nontrivial state | Construction order now matters | Either make it stateless, or make it the sole owner of that state |
| Users instantiate all N policies in every call site | You leaked the template into public API | Provide a named default alias |
| A policy's interface uses `virtual` | You have reinvented runtime polymorphism with extra syntax | Delete the policy; use an interface and virtual dispatch directly |
