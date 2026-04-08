# Modern C++ Replacements for Classic Loki Patterns⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​​​‌‍​​​‌​​​​‍​​​‌​​‌‌‍‌​​​‌‌‌​‍​​​​‌​​​‍‌‌​​‌​‌​⁠‍⁠

Load this file only when you are **writing or reviewing C++20/23 code** that translates a Loki-era pattern into modern syntax. Do not load it for high-level design questions — the design tradeoffs live in `SKILL.md`.

Each section shows the 2001 form (for recognition) and the modern form you should actually write. Every modern example is runnable on GCC 13+, Clang 17+, and MSVC 19.37+.

---

## 1. Tag dispatch → `if constexpr`

**2001 form** — one overload per tag, plus `Int2Type`:

```cpp
template <int N> struct Int2Type { enum { value = N }; };

template <class T> void impl(T& x, Int2Type<0>) { /* fast path   */ }
template <class T> void impl(T& x, Int2Type<1>) { /* safe path   */ }

template <class T> void doIt(T& x) {
    impl(x, Int2Type<std::is_trivial_v<T> ? 0 : 1>{});
}
```

**Modern form** — one body, branches elided after instantiation:

```cpp
template <class T> void doIt(T& x) {
    if constexpr (std::is_trivial_v<T>) {
        /* fast path */
    } else {
        /* safe path */
    }
}
```

**Gotcha:** The discarded branch is still parsed and must be well-formed *as a template*. If it references members that only exist on the other instantiation, wrap the reference in a dependent expression or split back into two overloads. `if constexpr` does NOT give you duck typing for free.

---

## 2. SFINAE probing → concepts

**2001 form** — `void_t` detection trait plus `enable_if_t`:

```cpp
template <class, class = void>
struct has_serialize : std::false_type {};

template <class T>
struct has_serialize<T, std::void_t<decltype(std::declval<T>().serialize())>>
    : std::true_type {};

template <class T>
auto save(const T& o) -> std::enable_if_t<has_serialize<T>::value> {
    o.serialize();
}
```

**Modern form** — one concept, one function:

```cpp
template <class T>
concept Serializable = requires(const T& t) {
    { t.serialize() } -> std::convertible_to<std::string>;
};

void save(const Serializable auto& o) { o.serialize(); }
```

**Why this matters:** When a user passes a type missing `serialize()`, the SFINAE version errors with a substitution-failure cascade that names no requirement. The concept version errors with `"constraint 'Serializable' not satisfied: 't.serialize()' would be ill-formed"`. Debugging time drops from 20 minutes to 20 seconds.

---

## 3. EBCO via inheritance → `[[no_unique_address]]` member

**2001 form** — inherit from policy to enable EBCO:

```cpp
template <class Alloc>
class vector : private Alloc {                // <-- inheritance
    T* begin_, *end_, *cap_;
    // access: static_cast<Alloc&>(*this).allocate(n)
};
```

**Problems this form has, in order of pain:**
1. `Alloc` might be `final` — fully legal since C++11 — silently collapsing EBCO and adding a pointer.
2. A member of `Alloc` might collide with a member of `vector`.
3. `private` inheritance leaks into the vocabulary of users who probe the type with traits.
4. Two empty policies of the same type can't share an address (unique-identity rule).

**Modern form** — member with the attribute:

```cpp
#if defined(_MSC_VER) && !defined(__clang__)
#  define POLICY_MEMBER [[msvc::no_unique_address]]
#else
#  define POLICY_MEMBER [[no_unique_address]]
#endif

template <class Alloc>
class vector {
    POLICY_MEMBER Alloc alloc_;
    T* begin_, *end_, *cap_;
};
```

**Verify shrinkage yourself** — never trust the optimization blindly:

```cpp
static_assert(sizeof(vector<int, std::allocator<int>>) == 3 * sizeof(void*));
```

Add one `static_assert` per supported stateless policy. When a user regresses it, the build fails at the exact line.

---

## 4. Recursive typelist → parameter packs

**2001 form** — `TypeList`, `Length`, `TypeAt` via recursion:

```cpp
template <class H, class T> struct TypeList {};
struct NullType {};

template <class L> struct Length;
template <> struct Length<NullType> { enum { value = 0 }; };
template <class H, class T>
struct Length<TypeList<H, T>> { enum { value = 1 + Length<T>::value }; };
```

**Modern form** — the pack *is* the list:

```cpp
template <class... Ts>
struct TypeList {
    static constexpr std::size_t size = sizeof...(Ts);
    template <std::size_t I>
    using at = std::tuple_element_t<I, std::tuple<Ts...>>;
};
```

**Why:** error messages no longer recurse. `TypeList<int, double, string>::at<1>` resolves in O(1) compile time with a one-line diagnostic on failure. The recursive form produces `Length<TypeList<int, TypeList<double, TypeList<string, NullType>>>>` in every error message.

---

## 5. CRTP + typelist scatter → variadic CRTP

**2001 form** — `GenScatterHierarchy<TypeList<Ts...>, Unit>` recurses on the list.

**Modern form** — variadic bases:

```cpp
template <class T> struct Holder { T value; };

template <class... Ts>
struct Scatter : Holder<Ts>... {};

// access: static_cast<Holder<double>&>(obj).value
```

The modern form is three lines and produces legible errors. The classic form is roughly 40 lines of recursive partial specializations.

---

## 6. `Select<B, T, F>::Result` → `std::conditional_t`

**2001 form:**

```cpp
template <bool B, class T, class F> struct Select        { using Result = T; };
template <class T, class F>          struct Select<false, T, F> { using Result = F; };
```

**Modern form:** `std::conditional_t<B, T, F>`. Ships in `<type_traits>` since C++14. No explanation needed. Delete the hand-rolled version.

---

## 7. Policy-based `SmartPtr` → `std::unique_ptr` or type-erased `std::shared_ptr`

The classic `Loki::SmartPtr<T, OwnershipPolicy, ConversionPolicy, CheckingPolicy, StoragePolicy>` carried five policy parameters. Its modern equivalents are:

- **Owning, single-owner, policy on deleter only:** `std::unique_ptr<T, Deleter>`. Deleter is a template parameter — the type-identity trap applies. Use a stateless deleter when you can; a *final* stateful deleter is a ticking footgun.
- **Owning, shared, deleter type-erased:** `std::shared_ptr<T>`. The deleter disappears into the control block at construction time, so `shared_ptr<Foo>` built with a custom deleter is interchangeable with any other `shared_ptr<Foo>`. This is the type-erasure tier of the type-identity remedy.
- **Non-owning observer:** `T*` or `std::observer_ptr<T>` (Library Fundamentals v2). Do not use `shared_ptr` or `weak_ptr` for non-ownership.

Writing your own `SmartPtr` in 2025 is almost always a mistake unless you are a runtime author with unusual cache or layout constraints. If you must, take the design lessons from Loki but write the implementation with `[[no_unique_address]]`, concepts for policy constraints, and a **non-virtual** destructor. Re-read the "NEVER put a virtual destructor on a policy-composed value type" entry in `SKILL.md` first.

---

## 8. `ScopeGuard` → `std::scoped_lock` + `std::unique_ptr` + `gsl::finally`

Alexandrescu's original `ScopeGuard` (2000) is obsolete in modern C++. Its uses split into:

- **RAII for already-modeled resources:** use the standard library (`unique_lock`, `scoped_lock`, `unique_ptr`, `ofstream`).
- **Ad-hoc cleanup:** use `gsl::finally([&]{ ... })` or write a five-line local class. Do not reach for Loki's `ScopeGuard`.

The original `ScopeGuard` predates lambdas. The techniques it pioneered are now built into the language.
