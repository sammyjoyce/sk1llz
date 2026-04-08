# Move Semantics & RAII — Deep Dive⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌​​​‌‍​​‌​​​​​‍‌​​​​‌​​‍​​​​‌​‌‌‍​​​​‌​​‌‍‌​​‌‌​​​⁠‍⁠

Load when: defining or debugging special member functions, Rule of Zero/Five questions, Pimpl compile errors, `noexcept` decisions, move-elision surprises, "my `vector` copies when it should move".

## Rule of Zero — the default you want

Delegate every owned resource to a member that already manages it. The compiler then generates correct copy/move/destroy operations with no risk of you getting them wrong.

```cpp
class Connection {
    std::unique_ptr<Socket> sock_;   // owns the socket
    std::string             host_;   // owns its buffer
    std::vector<Request>    queue_;  // owns its storage
public:
    Connection(std::string host);    // only constructors
    // No destructor. No copy. No move. The defaults are correct.
};
```

If you catch yourself writing `~Connection()`, stop and ask: "what resource here is not owned by a member?" The usual answer is "a raw pointer that should be a `unique_ptr`", or "a handle that needs a thin RAII wrapper". The rare legitimate cases: debugging hooks, unusual lifetimes, or custom deleters that don't fit a member type.

## Rule of Five — the escape hatch

If you declare **any** of the five — destructor, copy ctor, copy assign, move ctor, move assign — you must consider all five. The language suppresses the implicit moves if you declare a destructor or copy op, and suppresses the implicit copies if you declare a move op. Partial sets silently degrade: moves become copies, or copies disappear entirely.

Safe recipe when you must hand-write one: write, `= default`, or `= delete` for **all five** explicitly. Never leave some implicit and others explicit.

```cpp
class Unique {
public:
    Unique();
    ~Unique();
    Unique(const Unique&)            = delete;
    Unique& operator=(const Unique&) = delete;
    Unique(Unique&&) noexcept;
    Unique& operator=(Unique&&) noexcept;
};
```

## Why `noexcept` on the move constructor matters

`std::vector<T>`'s growth algorithm checks `std::is_nothrow_move_constructible_v<T>` at compile time via `std::move_if_noexcept`. If the move is not `noexcept`, it **copies** every element into the new buffer. This is because a throwing move mid-reallocation would leave both buffers partially destroyed — the standard chose "silently pessimize" over "break the strong guarantee".

A single non-`noexcept` member poisons the defaulted move of the whole class. Diagnose with:

```cpp
static_assert(std::is_nothrow_move_constructible_v<MyClass>);
static_assert(std::is_nothrow_move_assignable_v<MyClass>);
```

Other containers that rely on `noexcept` moves: `std::deque`, any container that implements strong-exception-safe resize.

## Moved-from state: "valid but unspecified"

The standard says a moved-from object is in a **valid but unspecified** state. Legal operations:

- **Safe:** destructor, assignment *to* the object, `clear()` / reset methods (`v.clear()`, `v = {}`, `s = ""`).
- **UB-risk:** any method that reads state (`size`, `empty`, `front`, `at`, iterators).

Library-specific guarantees exist for a handful of types:

| Type                  | Moved-from state guaranteed? |
|-----------------------|------------------------------|
| `std::unique_ptr<T>`  | Yes — `nullptr`              |
| `std::shared_ptr<T>`  | Yes — `nullptr`, `use_count()==0` |
| `std::optional<T>`    | Yes — `has_value()` unchanged (contained `T` is moved-from) |
| `std::vector<T>`      | **No** — likely empty but not required |
| `std::string`         | **No** — likely empty but not required (SSO may leave content) |

Never code against "it's empty after move" for vector/string. If you need that, write `x.clear()` after the move.

## `return std::move(local)` — the anti-pattern

```cpp
std::string make() {
    std::string s = build_it();
    return std::move(s);   // WRONG: disables NRVO, forces a move
}
```

NRVO constructs `s` directly into the caller's return slot — zero moves, zero copies. Adding `std::move` turns the return expression into an rvalue reference, which is **not an NRVO candidate**, so you get a mandatory (non-elidable) move.

**Exceptions where `std::move` on return is correct:**

```cpp
// Returning a by-value parameter — parameters are never NRVO candidates.
Result process(std::string input) {
    return std::move(input);   // correct
}

// Returning a member — members are never NRVO candidates.
std::string Container::take() && {
    return std::move(data_);   // correct
}
```

Compilers warn about the wrong case under `-Wpessimizing-move`; enable it.

## `std::move` on `const T` is a silent copy

```cpp
const std::string s = "hi";
std::vector<std::string> v;
v.push_back(std::move(s));   // COPIES. No diagnostic.
```

`std::move(s)` yields `const std::string&&`. `push_back(string&&)` cannot bind (const mismatch); `push_back(const string&)` can, so overload resolution silently picks copy. Never `const`-qualify a local you intend to move from. Some compilers warn under `-Wsuggest-override`-adjacent flags; none catch this reliably.

## Pimpl with `unique_ptr<Impl>`

The classic mistake:

```cpp
// widget.h
class Widget {
public:
    Widget();
    // no destructor declared — compiler emits implicit inline one
private:
    struct Impl;
    std::unique_ptr<Impl> p_;   // ERROR in every TU that destroys Widget
};
```

The implicit destructor is emitted inline in the header, where `Impl` is incomplete. `unique_ptr`'s default deleter needs `sizeof(Impl)` to call `delete`. Every TU that destroys a `Widget` fails with `sizeof of incomplete type`.

The fix — declare the destructor in the header, define it where `Impl` is complete:

```cpp
// widget.h
class Widget {
public:
    Widget();
    ~Widget();                          // declared
    Widget(Widget&&) noexcept;           // declared
    Widget& operator=(Widget&&) noexcept; // declared
private:
    struct Impl;
    std::unique_ptr<Impl> p_;
};

// widget.cpp
struct Widget::Impl { /* ... */ };
Widget::Widget()                                  : p_(std::make_unique<Impl>()) {}
Widget::~Widget()                                  = default;
Widget::Widget(Widget&&) noexcept                  = default;
Widget& Widget::operator=(Widget&&) noexcept       = default;
```

Rule of Five applies: if you declare the destructor, declare (and `= default`) the moves in the `.cpp` too, or they get suppressed.

Alternative: custom deleter type. If you can't move destructor definitions out of the header, use `std::unique_ptr<Impl, void(*)(Impl*)>` and define the deleter in the `.cpp`.

## RAII for non-memory resources

RAII is a *lifetime* technique, not a *memory* technique. Apply it to every resource:

- **Files:** `std::ifstream` / `std::ofstream` / `std::fstream` (closes in destructor), or `std::unique_ptr<FILE, int(*)(FILE*)>` with `fclose`.
- **Locks:** `std::scoped_lock` (multi-lock, deadlock-avoiding), `std::unique_lock` (movable, condvar-compatible), `std::lock_guard` (minimal, C++11).
- **Threads:** `std::jthread` — auto-joins in the destructor. `std::thread` `terminate`s on destruction if still joinable (an anti-pattern Stroustrup fought to correct with `jthread`).
- **OS handles:** wrap raw `int fd` or `HANDLE` in a thin class that invokes the right close function in its destructor and deletes copy.
- **C API refcounts:** `std::unique_ptr<T, decltype(&foo_release)>` with a custom deleter.

## `make_unique` vs `make_shared` — which to use

- **`std::make_unique<T>(args...)`:** exception-safe, preferred over raw `new`. Cannot take a custom deleter — use `std::unique_ptr<T, D>(new T(...), D{})` for that.
- **`std::make_shared<T>(args...)`:** one allocation for control block + object (vs. two for `shared_ptr<T>(new T)`). Fewer heap hits, better cache.
- **`make_shared` gotcha:** the object's *storage* is not freed until the last `weak_ptr` dies, even after the last `shared_ptr` goes away, because the control block shares the same allocation. For large objects with long-lived weak refs, this pins memory — use `std::shared_ptr<T>(new T{...})` to decouple.
- **`std::make_unique<T[]>(n)`** exists; `std::make_shared<T[]>` arrived in C++20.
- **Never** `std::make_unique<T>(new U)` — defeats exception-safety and leaks.

## Destructor rules

- **Noexcept by default** — the compiler infers `noexcept` for implicit destructors if all members' destructors are `noexcept`. Keep it that way.
- **Virtual iff polymorphic base** — `virtual ~Base() = default;` only if callers may `delete` through `Base*`. Otherwise leave non-virtual.
- **Protected non-virtual destructor** — communicates "you may not delete through this base". Useful for mixin/CRTP bases that aren't meant to be owned polymorphically.
- **Never throw** — during unwinding from another exception, throwing calls `std::terminate`. If cleanup can fail, expose an explicit `close()` users call before the destructor.

## `[[nodiscard]]` on constructors (C++20)

```cpp
class Lock {
public:
    [[nodiscard]] explicit Lock(std::mutex& m) : lock_(m) {}
private:
    std::lock_guard<std::mutex> lock_;
};
```

Without `[[nodiscard]]`, `Lock{m};` is a most-vexing-parse-adjacent bug — it creates a temporary that's destroyed immediately, releasing the lock instantly. `[[nodiscard]]` on the constructor makes the compiler warn.
