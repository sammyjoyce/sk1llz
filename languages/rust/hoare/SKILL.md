---
name: hoare-rust-origins
description: Design Rust APIs and low-level code around Hoare's original trade-offs: explicit ownership, explicit allocation, move-over-share, and safety without GC. Use when writing unsafe Rust, FFI, async or address-sensitive code, intrusive or graph-shaped data structures, or when borrow-checker friction suggests the ownership model is wrong. Triggers: unsafe, FFI, Pin, Rc, Arc, RefCell, lifetimes, aliasing, self-referential, graph, Send, Sync, arena.
---

# Hoare Rust Origins

Use this skill when the problem is about aliasing, address stability, or API shape. Do not use it for beginner syntax, trait boilerplate, or ordinary application code that already fits Rust's defaults.

## Load Only What Matters

- Before writing `unsafe` or custom interior mutability, READ the `std::cell::UnsafeCell` docs and the `std::ptr` safety section.
- Before writing self-referential, generator, or manual future code, READ `std::pin` and withoutboats' article `Pin`.
- Before designing C interop that stores pointers or callbacks, READ the Rustonomicon sections `Representing opaque structs` and `Asynchronous callbacks`.
- Do NOT load the Rust Book ownership chapters for these tasks; they are too introductory for the failure modes that matter here.

## Hoare's Original Bet

- Start from value-shaped APIs. Hoare's 2012 framing of Rust centered on value types, borrowing, move-vs-copy, and traits; if your design starts from shared ownership or ambient mutation, you are already swimming upstream.
- Treat allocation strategy as part of the semantics. Hoare explicitly called out dense heap-heavy layouts as a systems-language failure mode and cited common Java library structures with roughly 80% heap overhead; default to contiguous ownership and visible moves until measurement proves you need something else.
- Spend complexity only when it buys real safety. Hoare later argued that first-class references plus explicit lifetime variables may not "pay for themselves". When borrow checking dominates the design, first ask whether the API should become values-in and values-out instead of longer-lived borrows.

## Before You Code, Ask Yourself

- Do I need stable address or only stable identity?
  Stable identity usually means typed IDs, arenas, or generational indices. Stable address means allocate once, expose after pinning, and stop moving it.
- Is this borrow truly temporary?
  If it must live in a struct, iterator, future, cache, or callback registration, treat that as an ownership design problem, not a missing annotation problem.
- Am I sharing to avoid a clone, or because multiple actors genuinely co-own the value?
  Small clones are often the cheaper design than infecting an API with lifetimes or synchronized aliasing.
- Am I preserving an OO back-pointer layout out of habit?
  Rust tolerates trees far better than arbitrary pointer graphs.
- Is the compiler fighting aliasing or state modeling?
  Aliasing problems want ownership redesign. State problems want typestate, but only at narrow protocol boundaries with small state spaces.

## Decision Rules

- Need graph-like or cyclic structure:
  Prefer arena storage plus typed IDs. Use `Weak` only for real backedges. References inside nodes are a last resort.
- Need mutable shared state on one thread:
  Prefer one owner plus callbacks or message passing. Reach for `RefCell` as a boundary adapter, not the architecture.
- Need mutable shared state across threads:
  Prefer ownership transfer or sharded state. Use `Arc<Mutex<_>>` or `Arc<RwLock<_>>` only around narrow choke points.
- Need a returned reference:
  Ask whether returning owned data, a small clone, or an index would simplify the entire API. Hoare's own instinct was that most references should have stayed second-class parameter passing.
- Need a hot traversal API:
  Consider callback-style or internal iteration before exporting borrowed iterator state. Hoare preferred coroutine-style iteration partly because it avoided lifetime-heavy iterator objects and did not require inlining large amounts of library code.
- Need an FFI handle:
  Model an opaque `#[repr(C)]` type and pass raw pointers or handles. If the foreign resource is thread-affine or address-sensitive, prevent accidental `Send`, `Sync`, or `Unpin` on the Rust wrapper.

## Non-Obvious Constraints

- `Pin` does not make self-referential construction safe. It only makes already-address-sensitive values safe to manipulate after the point where their address matters.
- Async code has a real phase change: before first `poll`, a future may move freely; after self-references or borrowed state become part of the machine, movement becomes invalid. Design around that boundary instead of hand-waving it.
- `UnsafeCell` is the only legal escape hatch for mutation through shared access. It does not relax the uniqueness rules of `&mut`, and it does not make races acceptable.
- `UnsafeCell` can change layout by disabling niche optimizations. On 64-bit targets, `Option<NonNull<u8>>` is typically 8 bytes, while `Option<UnsafeCell<NonNull<u8>>>` grows to 16. Re-check ABI, FFI layout, and packed representations after introducing interior mutability.
- `NonNull<T>` is covariant. If your abstraction mutates through it or stores shorter-lived data behind it, you usually need an invariant marker such as `PhantomData<Cell<T>>`, not just a raw non-null pointer.
- `Rc::get_mut` requires no other `Rc` or `Weak` handles. `Rc::make_mut` will clone when other `Rc`s exist, but if only `Weak`s remain it may silently disassociate those `Weak`s instead. If observers rely on identity, clone-on-write can break the model you thought you had.
- Raw pointer validity is access-specific. A pointer can be acceptable for zero-sized operations and still be invalid for an actual read or write. Once you materialize a reference, pointer and reference accesses are no longer freely interleavable.
- `Vec` and `String` pointers are not stable across growth. If you mutate capacity after handing out interior pointers, you built a delayed use-after-free.

## NEVER Patterns

- NEVER reach for `Rc<RefCell<T>>` because it feels like the quickest way to recover OO object graphs. It is seductive because it suppresses borrow-checker pressure immediately. Instead it reintroduces runtime borrow panics, cycle leaks, and invisible temporal coupling. Instead keep one owning direction, use `Weak` or typed IDs for backedges, and concentrate mutation behind explicit phases.
- NEVER "fix" thread-sharing with `Arc<Mutex<T>>` at the top of the object graph because it compiles quickly. It is seductive because it preserves familiar shared-state designs. Instead it serializes unrelated work, hides lock ordering, and replaces compile-time alias proofs with runtime contention. Instead move ownership between tasks or shard the protected state.
- NEVER use `Pin` as borrow-checker duct tape because `Pin` does not grant safe self-reference construction. It is seductive because the name sounds like "make address stable". Instead you get unverifiable constructors or unsafe code with dangling self-pointers. Instead use arenas or typed IDs, or isolate the one address-sensitive object behind a tiny audited unsafe abstraction.
- NEVER cast or transmute `&T` into `&mut T`, or mutate through `NonNull` or raw pointers derived from shared references, because `UnsafeCell` is the only sanctioned opt-out from shared immutability. It is seductive because the data looks uniquely owned in your mental model. Instead the optimizer may assume the shared reference is immutable, which makes the code undefined. Instead thread mutability through `UnsafeCell` and obtain raw pointers via `.get()` or `&raw`.
- NEVER pass borrowed Rust references into C for retained or asynchronous use because the foreign side cannot uphold Rust lifetime, aliasing, or teardown guarantees. It is seductive because a borrowed pointer is easy to obtain. Instead you get use-after-free or callbacks into dead objects after `Drop`. Instead pass opaque handles or raw pointers, unregister in `Drop`, and guarantee no callbacks after deregistration.
- NEVER create raw pointers via an intermediate reference when memory may be packed, unaligned, or uninitialized. It is seductive because `&mut field as *mut _` looks idiomatic. Instead the reference creation itself can already be UB. Instead use `&raw mut` or `&raw const`, then pair with `read_unaligned` or `write_unaligned` where needed.
- NEVER store ordinary references or self-pointers inside data structures that may move because it feels cleaner than IDs. It is seductive because field access stays cheap. Instead any move, reallocation, or suspension boundary can invalidate the graph. Instead store owned data, `Weak`, or typed indices; pin only when stable address is a semantic requirement, not a convenience.

## Counterintuitive Best Practices

- Clone first, borrow second, when the clone removes a lifetime from the public API. Hoare's goal was practicality, not ascetic non-cloning.
- Prefer typed indices for graphs even when they feel less elegant than references. They are often easier to audit because identity and storage are explicit.
- Treat `unsafe impl Send` or `unsafe impl Sync` as a proof obligation, not a convenience. If you cannot state the invariant in one short paragraph, the unsafe surface is too large.
- Use typestate where invalid transitions are expensive and the state space is small: protocol handshakes, parser phases, builder completion. Do not blanket-typestate business objects or large async state machines.
- If you need an opaque FFI wrapper, copy the Nomicon pattern: private field, marker data, and explicit trait behavior. The marker is not decoration; it prevents accidental auto-traits on a handle the foreign side cannot actually support.

## Failure Recovery

- If the borrow checker resists the same design twice, stop adding lifetimes and redraw ownership on paper.
- If unsafe code needs more than one invariant paragraph, split the module until each unsafe block protects one claim.
- If shared mutability seems unavoidable, write down who may mutate, on which thread, and under what synchronization. If that list is vague, the design is not ready.
- If you need stable identity plus mutation plus cycles, use arenas with generation checks before you even consider self-referential structs.

Hoare-style Rust is not "never share". It is "make sharing expensive enough that you only do it when the semantics truly demand it."
