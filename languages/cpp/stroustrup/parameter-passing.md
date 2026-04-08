# Parameter Passing — Core Guidelines F.15–F.18⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌​​​‌​‍​‌‌‌​‌​‌‍​​‌‌‌​‌‌‍​​‌‌​‌‌‌‍​​​​‌​​‌‍‌​‌​​​​‌⁠‍⁠

The full decision matrix Stroustrup and Sutter codified. Load when you're writing a new signature, reviewing an API, arguing about `const T&` vs `T`, or chasing a hot-loop regression.

## The one-line rule

Pass **small trivially-copyable** types by value; everything else by reference (`const T&` for in, `T&` for in-out, `T&&` or value-and-move for sink). Use `unique_ptr` / `shared_ptr` in a signature only when transferring or sharing *ownership*.

## The master table

| Intent          | Parameter type                          | Examples                                             | Notes |
|-----------------|-----------------------------------------|------------------------------------------------------|-------|
| in, cheap       | `T`     (by value)                      | `int`, `double`, `Point2D`, `std::string_view`, `std::span<T>` | "Cheap" ≈ trivially copyable AND ≤ 2 machine words (16 bytes on 64-bit). |
| in, expensive   | `const T&`                              | `const std::string&`, `const std::vector<T>&`, `const Matrix&` | Default for non-trivial types when the function only reads. |
| in, optional    | `const T*` or `std::optional<T>`        | `const Widget*` where `nullptr` means "absent"       | Document the null/none contract explicitly. |
| in-out          | `T&`                                    | `std::string&`, `Buffer&`                             | Never `T*` when a value must exist — use a reference. |
| sink (will store) | `T` by value + `std::move` inside **or** `T&&` overload | `void push(T)` + `v_.push_back(std::move(x));` | By-value+move is usually clearer; use the rvalue-ref overload only when profiling demands. |
| out only        | Return it (`return T{...};`)            | Return `T`                                            | Out parameters via `T&` are obsolete. Trust RVO/NRVO. |
| forwarding      | `T&&` in a deduced template + `std::forward<T>` | `template<class T> void emplace(T&&)`       | Only valid when the `&&` applies to a deduced template parameter. |

## "Cheap to copy" — the subtle threshold

"Cheap" means trivially copyable **and** ≤ 2 machine words. Common traps:

- **`std::shared_ptr<T>`** is 16 bytes but its *copy* is not free — atomic refcount increment on every copy. Treat as "expensive in" and pass by `const shared_ptr<T>&` (or, preferably, by `const T&` if the function isn't sharing ownership).
- **`std::string`** is 24–32 bytes (SSO buffer inside); pass as `const string&` or `string_view`.
- **`std::string_view`** is 16 bytes, trivially copyable → pass **by value**.
- **`std::span<T>`** is 16 bytes, trivially copyable → pass **by value**.
- **`std::function<Sig>`** is large and has atomic state — pass as `const function&` or use a template `Callable`.
- **`std::array<int, 4>`** is 16 bytes, trivially copyable — by value is fine; the ABI passes it in registers on common platforms.

## Sink parameters: by-value-and-move vs rvalue-reference overload

### Recommended default: by value + `std::move`

```cpp
class Person {
    std::string name_;
public:
    explicit Person(std::string name) : name_(std::move(name)) {}
};
```

One signature handles both lvalues (one copy into the parameter + one move into the member) and rvalues (two moves, one elided). Simple, correct, exception-safe, no template bloat.

### Optimization: rvalue-reference overload (when profiled)

```cpp
class Person {
    std::string name_;
public:
    explicit Person(const std::string& n) : name_(n) {}        // 1 copy
    explicit Person(std::string&& n) : name_(std::move(n)) {}  // 1 move
};
```

One fewer move in the lvalue path. Use only when the extra move is measurably expensive. For `std::string` and `std::vector` a move is just pointer-swap — the by-value-plus-move version is almost always fine.

### Anti-pattern: `const T&` + make-a-copy inside

```cpp
explicit Person(const std::string& n) : name_(n) {}  // BAD for rvalues
```

Always copies, even when the caller passes a temporary. Use sink form.

## Forwarding references — templates only

`T&&` in a deduced template context is a **forwarding reference**; in any other context it's an **rvalue reference**.

```cpp
template <class T> void a(T&& x);        // forwarding — binds lvalue or rvalue
void b(std::string&& x);                 // rvalue reference — lvalues rejected
template <class T> void c(std::vector<T>&& x);  // rvalue reference — T deduced from element
```

Forwarding references **must** be paired with `std::forward<T>`, never `std::move` — using `std::move` on a forwarding reference silently steals from the caller's lvalue:

```cpp
template <class T>
void store(T&& x) {
    cache_.push_back(std::forward<T>(x));   // correct
 // cache_.push_back(std::move(x));         // WRONG — steals caller's lvalue
}
```

## Ownership in parameters — F.7, F.26, F.27

| You want…                                   | Take a…                                  |
|---------------------------------------------|------------------------------------------|
| Non-owning, must exist                      | `T&`                                     |
| Non-owning, optional                        | `T*` (nullable) or `std::optional<T&>` (C++26) |
| Shared ownership (function will co-own)     | `std::shared_ptr<T>` by value            |
| Observer of a shared_ptr (no co-ownership)  | `const T&` (take the dereferenced object) |
| Transfer of ownership                       | `std::unique_ptr<T>` by value            |

Anti-patterns:

- `const std::unique_ptr<T>&` — "I promise not to transfer." No — this lies about transfer and is just `const T&` with extra noise.
- `const std::shared_ptr<T>&` — "Avoids the refcount bump." Slightly defensible for hot loops, but usually `const T&` is the right signature: the function doesn't share ownership, it just reads.
- `std::shared_ptr<T>*` — no legitimate use.

## Return values

- **Return by value.** Trust RVO/NRVO. Stop using out parameters.
- **Return `T&`** only for accessors into data the function doesn't own (e.g., container `operator[]`).
- **Return `const T&`** for read-only accessors into member state.
- **Never return `auto&&`** from a non-forwarding function — it forwards references to locals.
- **Multiple returns:** return a named `struct` (preferred — names document intent) or `std::tuple<T...>` to be bound with structured bindings.

## Common pitfalls

- **`const T&` member init from a parameter whose lifetime ends at the end of the ctor:** e.g., storing a `string_view` built from a `const string&` parameter. The view's target is the caller's argument, which may be a temporary. Prefer owning storage.
- **`string_view` parameter constructed from a `string&&`:** the rvalue's lifetime ends at the end of the full-expression. Storing the `string_view` dangles; using it within the same statement is safe.
- **Default argument that is a temporary bound to `T&`:** `void f(T& x = T{});` — forbidden pre-C++20, surprising in C++20. Use overloads instead.
- **Passing `std::initializer_list<T>` by reference:** init lists are cheap (two pointers); pass by value.
- **"Universal reference" in a non-deduced context:** `void f(std::vector<int>&& v)` is **not** forwarding — it's an rvalue-only parameter. Forwarding requires the `&&` to apply to a directly-deduced template parameter.
