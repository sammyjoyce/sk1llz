---
name: stroustrup-cpp-style
description: Write modern C++ in Bjarne Stroustrup's idiom — RAII-first ownership, invariant-bearing classes, zero-overhead abstractions, and C++ Core Guidelines discipline. Use when writing or reviewing C++17/20/23 code, designing APIs or class hierarchies, choosing parameter-passing or ownership strategies, debugging move-semantics or lifetime bugs, deciding between exceptions and std::expected, writing concepts, fixing Pimpl compile errors, or enforcing Core Guidelines. Triggers include C++, Stroustrup, Core Guidelines, RAII, smart pointers, unique_ptr, shared_ptr, move semantics, rule of zero, rule of five, std::expected, concepts, ADL, hidden friends, noexcept, constexpr, string_view, span, GSL, not_null, pimpl.
---

# Stroustrup-Style Modern C++

## Before you type, ask yourself

1. **What invariant does this class enforce?** A class without an invariant should be a `struct`. If you cannot name the invariant in one sentence, you have not designed the class — you have grouped variables.
2. **Who owns every allocation, and when is it released?** The answer must be structural (a `unique_ptr` member, a container, a stack object), never "the caller remembers".
3. **Can the compiler reject misuse?** If "don't call `close()` twice" is a documented rule, the type is wrong. Make the illegal state unrepresentable.
4. **What does this cost when I'm wrong about it being cheap?** Zero-overhead means "no cost vs. hand-written C", not "free". Know the cost.

If you can't answer (1) and (2) before typing, stop and redesign.

## Expert heuristics (hard-won, non-obvious)

### Rule of Zero is the default; a hand-written destructor is a smell
Delegate every resource to a member that already owns it (`unique_ptr`, `vector`, `string`, `jthread`, `scoped_lock`). The compiler then generates correct copy/move/destroy for free. **If you catch yourself writing `~Foo()`, ask "what resource is leaking through the cracks?"** — the fix is almost always "wrap this raw pointer" or "use a container." Rule of Five is the escape hatch, not the goal.

### `{}` initialization is about narrowing, not style
`int x{3.14};` is a compile error (good). `int x(3.14);` silently truncates to `3`. Prefer `{}` EXCEPT two traps:
- `std::vector<int> v{10, 20};` makes a 2-element vector — `initializer_list<int>` wins overload resolution. Use `(10, 20)` for "10 copies of 20".
- `auto x{1};` gave `initializer_list<int>` in C++11–14; C++17 fixed single-brace to `int`. Never `auto x{a, b};`.

### `std::move` is a cast; `std::move` on `const` is a silent copy
`std::move(x)` yields an rvalue reference; whether a move actually happens depends on the destination. `std::move` of a `const T` produces `const T&&`, which cannot bind to a move constructor — the copy constructor is chosen with no diagnostic. **Never `const`-qualify a local you intend to move from.**

### Never `return std::move(local);`
NRVO already constructs the result directly in the caller's slot — zero moves. Writing `return std::move(x);` turns the return expression into an rvalue reference, which is not an NRVO candidate, so you get a mandatory move. The only correct uses of `std::move` in `return`: returning a by-value *parameter*, or returning a *member* — neither is NRVO-eligible.

### A moved-from object is "valid but unspecified" — not "empty"
After `std::move(x)`, only these are safe on `x`: destruction, assignment *to* `x`, and reset methods (`x.clear()`, `x = {}`). Do **not** read state: `x.size()`, `x.empty()`, `x.front()` may return anything. `std::unique_ptr` is a special case (spec guarantees `nullptr`); `vector`/`string` are **not**.

### `noexcept` on the move constructor doubles `vector` growth speed
`std::vector<T>` checks `std::is_nothrow_move_constructible_v<T>` at reallocation. If false, it **copies** every element to preserve the strong exception guarantee. One non-`noexcept` member poisons the entire defaulted move. Protect your classes with `static_assert(std::is_nothrow_move_constructible_v<MyClass>);` — silent pessimization is the worst kind.

### Thread-safe `const` requires a `mutable` mutex
A `const` method must be safe for concurrent callers. If it needs a lock, the `std::mutex` member is `mutable`. This is idiomatic, not a hack — `mutable` exists precisely for memoization caches and internal synchronization.

### `std::string_view` is not a free `const std::string&`
- Dangles if built from a `string` temporary held across statements. Microsoft classifies ~23% of C++ security bugs as lifetime-related; `string_view` misuse is the leading new source.
- Not null-terminated — `.data()` is **not** a C string; passing to C APIs is UB.
- `sizeof(string_view) == 16` on 64-bit; for very short strings, `const string&` can be faster.
- **Never** store `string_view` (or any range `view`) as a class member unless the owning data's lifetime is externally guaranteed. Store `string`, take `string_view` in parameters.

### Pimpl with `unique_ptr<Impl>` requires an out-of-line destructor
`unique_ptr<Impl>`'s implicit destructor needs `Impl` complete at the point of instantiation. Forward-declare in the header, then in the `.cpp`:
```cpp
// widget.h
class Widget { public: Widget(); ~Widget();  // declared
private: struct Impl; std::unique_ptr<Impl> p_; };
// widget.cpp — where Impl is complete
Widget::~Widget() = default;
```
Skipping the out-of-line `= default` gives cryptic "`sizeof` of incomplete type" errors in every TU that destroys a `Widget`. Same applies to move operations if you declare them.

### Concepts express *semantics*, not syntax
A concept that merely checks "has `operator+`" is worse than no constraint — it lies about intent (string concatenation? integer addition? pointer arithmetic?). A good concept names a property with an axiom you can state in one sentence (`Monoid`, `RandomAccessRange`, `TotallyOrdered`). If you can't write the axiom, keep the template unconstrained with a comment — don't name a shape check.

### `reinterpret_cast` is almost never what you want
For type punning use `std::bit_cast<T>(x)` (C++20, constexpr, zero cost). For raw byte copy use `std::memcpy`. `reinterpret_cast` between unrelated pointer types violates strict aliasing and is UB in most cases. Well-defined uses are narrow: to/from `std::byte*`/`char*`/`unsigned char*`, to/from `std::uintptr_t`, and platform-specific function/object pointer conversions.

### Exceptions for *exceptional*; `std::expected` for *expected*
Throw on: resource-acquisition failure, broken invariant, out-of-memory. Return `std::expected<T, E>` (C++23) or `std::optional<T>` for: parse failure, lookup miss, user-input validation — anywhere the caller is expected to handle it on the normal path. **Never mix both for the same category of failure** — callers can't reason about two error paths. Error codes are a C-ABI boundary tool only; they fight RAII because cleanup becomes manual.

### Strong types belong where two units share a representation
Replace a primitive with a strong type **when two units share the same representation and confusing them compiles silently**: `Milliseconds` vs `Seconds`, `UserId` vs `OrderId`, `Celsius` vs `Fahrenheit`. Don't wrap a primitive with no confusable sibling — that's bureaucracy. `std::chrono::duration` and `enum class` are the cheapest implementations; a 3-line wrapper class is the next step up.

## NEVER (seductive reason → concrete consequence → correct path)

- **NEVER use `new`/`delete` in application code.** Feels direct and explicit. *Consequence:* any exception between `new` and `delete` leaks, and the leak is invisible in review. *Do:* `std::make_unique<T>(...)` and let RAII handle unwinding.
- **NEVER expose a raw owning pointer in an API.** "It's just a pointer." *Consequence:* reviewers cannot tell ownership from observation, grep cannot find leaks, callers can't reason about lifetimes. *Do:* `std::unique_ptr<T>` for transfer; `T&` for required non-owning; `T*` only for nullable non-owning, and annotate `gsl::not_null<T*>` when it must not be null.
- **NEVER default to `std::shared_ptr`.** Feels "safe". *Consequence:* atomic refcount traffic on every copy, cycles that silently leak, and obscured ownership that makes refactoring impossible. *Do:* default to `unique_ptr`; promote to `shared_ptr` only when multiple owners genuinely coexist in time.
- **NEVER inherit publicly to reuse code.** Avoids duplication with one keyword. *Consequence:* Liskov violations, tight coupling, virtual-dispatch cost, slicing on by-value copy. *Do:* composition by default; inherit publicly only to model "is-a" polymorphism, and make the base destructor either `virtual` or `protected` non-virtual.
- **NEVER throw from a destructor.** Cleanup can legitimately fail (flush, close). *Consequence:* during stack unwinding from another exception, throwing calls `std::terminate` with no cleanup. *Do:* log/set a flag in the destructor; expose an explicit `close()` users call before destruction when they care about the outcome.
- **NEVER capture by reference (`[&]`) in a lambda that outlives the enclosing scope.** Works perfectly in the demo. *Consequence:* UB when the lambda runs after the frame is gone — async callbacks, `std::thread`, coroutines, event loops. *Do:* capture by value, or by `shared_ptr` copy when you need shared mutable state.
- **NEVER leave a single-argument constructor implicit.** `f(42)` reads nicely. *Consequence:* silent conversions cause overload ambiguities and surprise temporaries that bind to `const T&` parameters. *Do:* mark it `explicit`. For templates, `explicit(condition)`.
- **NEVER mark every function `noexcept`.** Feels like "free optimization". *Consequence:* a single thrown exception from inside calls `std::terminate` with no unwind — worse than crashing. *Do:* `noexcept` on moves, swaps, destructors, and functions whose specification guarantees no throw. Leave the rest silent.
- **NEVER use a macro where `constexpr`, `consteval`, `inline`, or a template works.** Brevity. *Consequence:* no scoping, no types, textual-substitution bugs, diagnostics that point into expanded code. *Do:* in 2024 C++ the only legitimate macros are include guards and platform feature detection.
- **NEVER store `string_view` or a range view as a class member.** It's cheap and non-owning. *Consequence:* dangling references that survive code review because they manifest weeks later under different memory layouts. *Do:* store the owning type; take views only in parameters.
- **NEVER return `auto&&` from a non-forwarding function.** Reads like "perfect return forwarding". *Consequence:* it forwards a reference to a local — dangling. *Do:* return `auto` (by value) or `decltype(auto)` only in forwarding wrapper templates.

## Decision trees

### Error handling
```
Can the caller meaningfully handle it?
├── No (bug, broken invariant) ............ assert / contract; no return path
├── Yes, rare and exceptional ............. throw a typed exception
├── Yes, routine and expected ............. std::expected<T, E> or std::optional<T>
└── Crossing a C ABI boundary ............. error code; document; wrap in C++ immediately
```

### Smart-pointer choice
```
Who owns this resource?
├── Single owner, clear lifetime .......... std::unique_ptr<T>   ← the default (~99% of cases)
├── Genuinely shared ownership ............ std::shared_ptr<T>
├── Observer of a shared_ptr .............. std::weak_ptr<T>
└── Non-owning reference .................. T& or gsl::not_null<T*>
```
If your first instinct is `shared_ptr`, pause. Ask: "could a `unique_ptr` at the right layer serve everyone else by reference?" — 9 times out of 10, yes.

### Class vs struct
```
Is there an invariant between members?
├── Yes ................ class, private data, constructors establish it
└── No ................. struct, public data, no constructors beyond aggregate
```

### Parameter passing (full table in `parameter-passing.md`)
```
Size ≤ 2 words and trivially copyable?  → pass by value
Large, read-only input?                 → pass by `const T&`
Sink (function will store/move it)?     → pass by value + `std::move` inside
In-out, must exist?                     → pass by `T&`
Optional in-out?                        → pass by `T*`, document the null contract
```

## Progressive loading

This file is enough for day-to-day writing and review.

- **Load `parameter-passing.md`** when: designing a new function signature, reviewing an API, arguing `const T&` vs `T` vs `T&&`, or chasing a call-site regression. Contains the full F.15–F.18 decision matrix and sink-parameter trade-offs.
- **Load `move-and-raii.md`** when: defining or debugging special member functions, Rule of Zero/Five questions, Pimpl compile errors, `noexcept` decisions, move-elision surprises, or "why did my `vector` copy instead of move?".
- **Load `generic-programming.md`** when: writing a template, designing a concept, seeing cryptic template errors, hidden-friend or CPO/ADL questions, or deciding between `if constexpr`, SFINAE, and concepts.
- **Load `philosophy.md`** only when you need the *why* — justifying an architectural decision, writing a team style guide, onboarding. Do **not** load for day-to-day coding.
- **Do NOT load `references.md`** for implementation tasks. It is an index of external resources (books, talks, papers), not a guide.
