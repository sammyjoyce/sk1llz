# Templates, Concepts & Generic Programming

Load when: writing a template, designing a concept, facing cryptic template errors, hidden-friend/CPO/ADL questions, choosing between `if constexpr`, SFINAE, and concepts.

## The Stroustrup view

Templates are for **generic programming** — writing code once that works for a family of types related by a *semantic contract*. They are **not** meant to be a compile-time functional programming language. Heavy TMP ("sub-languages in the type system") is a failure mode; reach for `constexpr`, `consteval`, `if constexpr`, or concepts instead.

## Concepts: semantics, not syntax

A good concept names a property that has an **axiom** — a one-sentence mathematical rule it satisfies. A concept that only checks syntactic shape is worse than no constraint, because it lies about intent.

### Bad — checks syntax only

```cpp
template <class T>
concept Addable = requires(T a, T b) { a + b; };
```

This accepts `std::string + std::string` (concatenation), `int + int` (arithmetic), `pointer + int` (pointer arithmetic), `std::chrono::duration + duration` (time arithmetic). A generic algorithm templated on `Addable` is correct only by accident — it has no idea which semantics it's getting.

### Good — names a semantic contract

```cpp
template <class T>
concept Monoid = std::regular<T> && requires(T a, T b) {
    { a + b } -> std::same_as<T>;
    { T{} }   -> std::same_as<T>;
    // Axiom (documented, not checked by the compiler):
    //   (a + b) + c == a + (b + c)       -- associative
    //   a + T{} == a == T{} + a          -- identity
};
```

The template's contract now has a name and a meaning. **If you can't write the axiom in one sentence, you don't have a concept — you have a shape check.** Keep the template unconstrained with a comment in that case; don't name a lie.

### Use standard concepts when they fit

Prefer library-provided concepts — they come with published axioms and every reader already knows what they mean:

- `std::regular` — copyable, default-constructible, equality-comparable with value semantics.
- `std::equality_comparable`, `std::totally_ordered`, `std::three_way_comparable`.
- `std::invocable<F, Args...>`, `std::predicate<F, Args...>`.
- `std::ranges::range`, `std::ranges::random_access_range`, `std::ranges::sized_range`.
- `std::integral`, `std::floating_point`, `std::signed_integral`.

## Hidden friends — the ADL-correct way to define operators

Define operators and customization points as `friend` functions **inside** the class body:

```cpp
class Path {
    std::string s_;
public:
    friend bool operator==(const Path& a, const Path& b) { return a.s_ == b.s_; }
    friend auto operator<=>(const Path& a, const Path& b) = default;
    friend std::ostream& operator<<(std::ostream& os, const Path& p) { return os << p.s_; }
    friend void swap(Path& a, Path& b) noexcept { std::swap(a.s_, b.s_); }
};
```

Benefits (each one matters in real codebases):

1. **Found only by ADL.** Unrelated `operator==` calls in the same TU don't consider this one. Faster compiles. Cleaner error messages when overload resolution fails.
2. **Cannot be called by implicit conversions on the first argument.** `operator==(path1, "string")` works; `operator==("string", "string")` is not intercepted.
3. **No namespace pollution.** The function lives in the enclosing namespace but is invisible to ordinary lookup — only ADL finds it.
4. **Natural access to private members** without declaring the operator outside the class.

Use hidden friends for: `operator==`, `<=>`, `swap`, stream inserters (`<<`, `>>`), arithmetic operators when needed, customization points that must be found by ADL.

## ADL and the `std::swap` two-step

`std::swap(a, b)` works only because of the idiom:

```cpp
using std::swap;     // make std::swap visible in unqualified lookup
swap(a, b);          // unqualified call — ADL finds user's swap first, falls back to std::swap
```

If you write `std::swap(a, b)` fully qualified, you bypass the user's `swap` and get the generic triple-move (which may be pessimal or incorrect for types that have a dedicated swap).

Modern C++ uses **Customization Point Objects (CPOs)** — function objects in `std::ranges` that perform the two-step for you:

```cpp
std::ranges::swap(a, b);       // correct — does the two-step internally
std::ranges::begin(range);     // correct — finds user's begin() via ADL
```

When you design a library with a customization point, make it a CPO (function object) or a `tag_invoke`-style dispatcher, not a plain function template. CPOs cannot be accidentally ADL-bypassed by callers.

## Template errors: definition-time vs instantiation-time

Pre-C++20, template errors happened at *instantiation*, often 200+ lines deep into `<iterator>`. Concepts move the check to the point of definition:

```cpp
template <std::ranges::input_range R>
void process(R&& r);

process(42);   // error: constraint not satisfied — AT THE CALL SITE
               // (not "no matching overload" after iterator_traits noise)
```

**Always constrain templates with concepts where a contract exists.** Unconstrained templates are a last resort for genuinely unconstrained code (type-erasure implementation details, low-level primitives).

## `if constexpr` vs SFINAE vs concepts — decision table

| Situation                                        | Use                      |
|--------------------------------------------------|--------------------------|
| Select overload by type property                 | Concepts (`requires`)    |
| Compile out a branch inside one function         | `if constexpr`           |
| Detect presence of a member, dispatch to impl    | Concepts + overload      |
| C++17 or earlier, need type-based dispatch       | SFINAE (`std::enable_if`) |
| Want cleanest error messages                     | Concepts                 |
| Tagged dispatch on multiple traits               | Concepts + `requires`    |

**Never mix SFINAE and concepts on the same overload set.** Overload resolution rules differ and the interaction is undefined-ish — different compilers pick different overloads. Commit to one approach per overload set.

## Two-phase name lookup — the portability pitfall

In a template, the compiler does lookup in two phases:

1. **At definition time**, all non-dependent names are looked up in the definition context.
2. **At instantiation time**, dependent names (those that depend on a template parameter) are looked up in the definition context *plus* ADL of the instantiation argument types.

```cpp
template <class T>
void f(T x) {
    helper(x);     // dependent on T — resolved at instantiation + ADL on T's namespace
    other(42);     // non-dependent — MUST be visible at the point of f's definition
}
```

MSVC historically delayed all lookup to instantiation (a long-standing standard violation), letting bad code compile. If you write cross-platform library templates, always compile-test with clang or gcc before shipping. A missing `#include` can hide in an MSVC-only codebase for years.

To force a name to be treated as dependent (and thus looked up at instantiation):

```cpp
template <class T>
void f(T x) {
    typename T::iterator it;       // 'typename' — T::iterator is a type
    x.template method<int>();      // 'template' — method is a template
}
```

## Forwarding references — templates only

`T&&` is a forwarding reference **only** when `T` is a directly-deduced template parameter:

```cpp
template <class T>
void a(T&& x);                       // forwarding — binds lvalue or rvalue

void b(std::string&& x);             // rvalue reference — lvalues rejected

template <class T>
void c(std::vector<T>&& x);          // rvalue reference — T deduced via vector, not x

template <class T>
void d(const T&& x);                 // rvalue reference — 'const' breaks forwarding
```

Always pair a forwarding reference with `std::forward<T>`. **Never** use `std::move` on a forwarding reference — it unconditionally converts to rvalue, silently stealing from the caller's lvalue:

```cpp
template <class T>
void store(T&& x) {
    cache_.emplace_back(std::forward<T>(x));   // correct
 // cache_.emplace_back(std::move(x));         // WRONG — steals lvalues
}
```

## Tips and gotchas

- **Prefer `std::ranges::iterator_t<R>` over `typename R::iterator`** — the ranges trait handles non-member iterators (C arrays, `ranges::views`) that don't have a nested `iterator` typedef.
- **Use `auto` return type for short templates** unless the deduced type would surprise the reader; then use a trailing return with `-> decltype(expr)`.
- **Constrain return types with concepts:** `auto f() -> std::integral auto` documents intent at the signature.
- **Don't put template bodies in `.cpp`** unless you also explicitly instantiate each specialization in that `.cpp`. "Export templates" was removed from the language in C++11.
- **`decltype(auto)`** — use in forwarding wrappers that need to preserve reference-ness of the return. Not for ordinary functions; `auto` is almost always what you want.
- **CTAD (C++17):** class template argument deduction — `std::pair p{1, 2.0};` deduces `std::pair<int, double>`. Write a deduction guide when the defaults mis-deduce (especially for types that store references).
- **Concept-constrained `auto` in function parameters (abbreviated templates, C++20):** `void f(std::integral auto x)` is shorthand for `template <std::integral T> void f(T x)`. Good for one-off templates; use full form for anything non-trivial.
