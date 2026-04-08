---
name: sutter-exceptional-cpp
description: "Write and review modern C++ in Herb Sutter's style: explicit failure contracts, ownership semantics, polymorphic boundaries, and concurrency-safe const behavior. Use when code involves exception safety, `noexcept`, moved-from state, `const` plus `mutable`, `pimpl`, `clone`, smart pointers, virtual interfaces, or lock-sensitive callbacks. Triggers: exceptional c++, strong guarantee, basic guarantee, commit or rollback, NVI, `make_unique`, `make_shared`, `weak_ptr`, `noexcept`, `shared_ptr`, `unique_ptr`."
tags: exceptions, noexcept, const, concurrency, move-semantics, pimpl, polymorphism, smart-pointers
---

# Sutter Exceptional C++

This skill is for code that must stay correct when construction, allocation, callbacks, or concurrency go wrong. It is not a generic C++ style guide.

Do not load generic "clean code" advice for this task. The hard part here is preserving invariants across failure paths and extension points.

## Start Here

Before changing a type, classify it:

- Value type: copyable, equality-preserving, swappable, ordinary moved-from state.
- Unique owner or handle: move-first, copy maybe deleted, resource lifetime explicit.
- Polymorphic interface: public virtual surface only when dynamic dispatch is the product; otherwise prefer NVI.
- ABI firewall: `pimpl` or opaque handle where compilation boundaries matter more than convenience.

Before writing code, ask yourself:

- What is the promised failure contract for each mutating operation: `noexcept`, strong, or basic?
- Which operations may execute unknown code: virtual dispatch, templates on `T`, comparators, hashers, allocators, deleters, logging sinks, callbacks?
- Can `const` callers race each other? If yes, is the object truly read-only or internally synchronized?
- Will a moved-from object still satisfy every invariant that observers assume?

## Sutter Heuristics

- Default to the rule of zero. A user-declared destructor or copy operation usually means you just took responsibility for move generation, exception guarantees, and self-assignment behavior too.
- Promise `noexcept` only when you can preserve invariants without fallback allocation, locking, or validation work. `noexcept` is a performance contract to the standard library; a false promise turns an ordinary failure into `std::terminate`.
- If a type is copyable, treat move as an optimization of copy, not a semantic escape hatch. Generic code exploits move aggressively only when it can trust it not to throw.
- A moved-from object is still a normal live object. "Valid but unspecified" never means "destroy-only." If `operator<`, `swap`, or reassignment break after move, the type is broken.
- If move would violate a non-null or non-empty invariant, encode that invariant in the type or suppress move. Do not let a defaulted move silently manufacture states the rest of the interface cannot tolerate.
- `const` on shareable objects means "read-only or internally synchronized." Sutter's M&M rule applies: `mutable` and `mutex` or `atomic` should travel together.
- Prefer public nonvirtual functions plus private or protected virtual hooks when a base class must enforce invariants, locking, or exception translation. Public virtuals are harder to defend because overrides can skip the guardrails.
- Prefer value semantics unless you are expressing ownership or polymorphic lifetime. `shared_ptr` is not a harmless default; it adds control-block cost, weak-cycle design pressure, and cross-thread refcount traffic.
- Use `pimpl` with `unique_ptr`, not `shared_ptr`, unless sharing the implementation object is the semantic goal. `shared_ptr` silently changes copy semantics and pays for refcounting even when nobody shares.
- If deep copy of a polymorphic object is required, delete public base copy and move and add `clone`; otherwise callers will eventually slice.

## Failure-Contract Decision Tree

When a mutating operation can fail:

- Need caller-visible rollback: stage all throwing work off to the side, then commit with `noexcept` swap or pointer flip.
- Need peak performance on hot equal-capacity assignment: reuse storage in place, document that you only provide the basic guarantee, and test self-assignment plus self-move.
- Need both rollback and reuse: split the operation into reserve or prepare and commit steps; do not fake both with a single clever assignment operator.

When choosing smart-pointer factories:

- Multiple allocations appear in one full expression: use `make_unique` or `make_shared` so construction and ownership become one non-interleaved operation.
- Shared lifetime is real and weak observers are short-lived: prefer `make_shared` for one allocation and better locality.
- Shared lifetime is real but weak observers can outlive a large object, you need a custom deleter, or you are adopting an existing pointer: construct `shared_ptr` without `make_shared`.

When choosing interface shape:

- Concrete or value type: no virtual assignment, no public data races, swap should be cheap and `noexcept`.
- Polymorphic base: destructor must be public virtual or protected nonvirtual; public copy and move should usually be deleted.
- Pimpl or firewall: destructor and any assignment operator that may destroy the impl must be defined out of line where the impl is complete.

## NEVER Rules

- NEVER write `f(unique_ptr<T>{new T}, unique_ptr<U>{new U})` because the smart pointers do not exist until after construction finishes, and argument evaluation can interleave and leak on exception. Instead use `make_unique`-style factories at the call site.
- NEVER add `noexcept` "for STL performance" when move or swap might allocate, lock, or validate because the promise is seductive and benchmarks may improve, but a late throw becomes `std::terminate`. Instead use conditional `noexcept` or let generic code copy.
- NEVER assume copy-and-swap is always the best assignment pattern because the strong guarantee is seductive, but on large equal-sized buffers it can be an order of magnitude slower and defeats storage reuse. Instead reserve copy-and-swap for rollback-critical paths and write an in-place basic-guarantee assignment when reuse dominates.
- NEVER leave moved-from objects semantically poisoned because "valid but unspecified" sounds like a license to drop invariants. The concrete consequence is algorithms that compare, swap, or reassign moved-from objects crash far from the bug. Instead leave a normal invariant-preserving state, often default-like.
- NEVER use `mutable` as a concurrency loophole because old pre-C++11 "logical const" intuition is seductive. Unsynchronized caches inside `const` functions create data races that only appear under load. Instead pair mutable state with a mutex or atomic, or remove the cache.
- NEVER expose public copy or move on a polymorphic base because it feels uniform with concrete types, but it slices and destroys substitutability. Instead delete the public operations and offer `clone` only if deep copy is required.
- NEVER reach for `shared_ptr` to avoid making an ownership decision because it feels reversible. The consequence is refcount churn, accidental cycles, and with `make_shared` the object's storage can stay pinned until the last `weak_ptr` dies. Instead default to `unique_ptr` plus raw or reference observers.
- NEVER call virtual functions, callbacks, comparators, template customization points, or other unknown code while holding a lock or while invariants are temporarily broken. It is seductive because the call site looks local, but the real consequence is deadlock, or even single-threaded reentrant observation of half-mutated state. Instead snapshot what you need, unlock, then call out.

## Patterns To Apply

### Assignment

- For ordinary value types, let the compiler generate copy and move if members already do the right thing.
- For hand-written assignment, decide explicitly between strong guarantee via temp-and-swap and basic guarantee via storage reuse.
- Self-assignment and self-move should be correct by construction, not rescued by defensive branches unless measurement justifies them.

### Const Plus Concurrency

- A `const` member may lock; the lock object should itself be `mutable`.
- A `const` member may update cached data only if callers cannot observe a race and the cache update is synchronized.
- If you cannot make a `const` observer race-free, it is not really `const` in Sutter's sense.

### Pimpl

- `unique_ptr<impl>` is the default.
- Define the destructor out of line.
- If copy or move assignment can destroy the old impl, define those out of line too; incomplete-type friendliness ends exactly where deletion begins.

### Polymorphic APIs

- Public interface establishes preconditions, postconditions, locking, and exception policy.
- Virtual hook does only the customization work.
- If callers own through base pointers, deletion must be correct even when they use `unique_ptr<Base>`.

## Review Checklist

Before finishing, verify:

- Every mutating function has an intentional guarantee: `noexcept`, strong, or basic.
- Every moved-from state still obeys invariants.
- Every `const` function on a shareable type is race-free.
- Every ownership edge is explicit: unique, shared, or observing.
- Every virtual boundary preserves invariants and deletion semantics.
- Every lock scope excludes unknown code.

If the code still relies on "callers will remember not to do that," it is not Sutter-grade yet.
