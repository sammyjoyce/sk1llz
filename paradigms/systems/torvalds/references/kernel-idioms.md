# Kernel C idioms — what every line of kernel code assumes you know⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​​‌‌​‌‍‌‌‌‌​‌‌‌‍​‌​‌​​​‌‍‌‌‌‌​​‌​‍​​​​‌​‌​‍‌‌​‌‌‌‌‌⁠‍⁠

Load this when you are about to write or review code that does multiple allocations, returns errors from a function that also returns a pointer, embeds itself into a generic data structure, or chooses an allocation flag.

## 1. The goto cleanup ladder (the *only* correct error pattern)

```c
int probe_device(struct device *dev)
{
    struct mydev *m;
    int ret;

    m = kzalloc(sizeof(*m), GFP_KERNEL);   /* sizeof(*m), not sizeof(struct mydev) */
    if (!m)
        return -ENOMEM;

    m->buf = kmalloc(BUF_SIZE, GFP_KERNEL);
    if (!m->buf) {
        ret = -ENOMEM;
        goto err_free_m;
    }

    m->wq = alloc_workqueue("mydev", 0, 0);
    if (!m->wq) {
        ret = -ENOMEM;
        goto err_free_buf;
    }

    ret = request_irq(dev->irq, mydev_isr, 0, "mydev", m);
    if (ret)
        goto err_destroy_wq;

    dev_set_drvdata(dev, m);
    return 0;

err_destroy_wq:
    destroy_workqueue(m->wq);
err_free_buf:
    kfree(m->buf);
err_free_m:
    kfree(m);
    return ret;
}
```

Rules that are not negotiable:

- **Labels are named for what the *next* statement frees**, not what just failed. `err_free_buf` frees `buf`. This means you can insert a new step between two existing ones without renaming any labels.
- **Unwind in exact reverse order of acquisition.** Anything else leaks or double-frees under the wrong combination of failures.
- **One `return` for success, one `return ret` at the bottom for failure.** Multi-return functions with allocations are how leaks ship.
- **Never `goto` *forward* and never across a label that already unwound something.** Forward gotos defeat the entire pattern.

If a function has only one allocation, the ladder is overkill — just `kfree(p); return ret;`. The ladder appears starting at the second resource.

## 2. ERR_PTR / PTR_ERR / IS_ERR — encoding errors in pointer return values

A function that "returns a pointer or fails" cannot use `NULL` for failure if `NULL` is a legitimate result, and even when it isn't, callers want to know *why* it failed. The kernel encoding:

```c
struct foo *foo_create(int x)
{
    struct foo *f;

    if (x < 0)
        return ERR_PTR(-EINVAL);

    f = kzalloc(sizeof(*f), GFP_KERNEL);
    if (!f)
        return ERR_PTR(-ENOMEM);

    return f;
}

/* caller */
f = foo_create(x);
if (IS_ERR(f))
    return PTR_ERR(f);   /* propagate the errno */
```

`ERR_PTR(-E)` packs `-E` into the top of the address space (the last 4096 addresses), which can never be a valid kernel pointer. `IS_ERR()` checks that range; `PTR_ERR()` extracts the errno.

**The trap:** `if (!f)` does *not* catch `ERR_PTR` returns. A function returning `ERR_PTR(-ENOMEM)` followed by `if (!f) goto err` will fall through to success and crash on first dereference. Pick *one* convention per function and document it. Most kernel functions either always-`NULL`-on-fail OR always-`ERR_PTR`-on-fail; mixing is a bug.

## 3. container_of — generic data structures without templates

```c
#define container_of(ptr, type, member) ({                      \
    void *__mptr = (void *)(ptr);                               \
    ((type *)(__mptr - offsetof(type, member))); })
```

Read it as: "given a pointer to a `member` field that lives inside a `type` struct, give me the pointer to the enclosing `type`". It works because struct field offsets are compile-time constants.

The pattern this enables: **embed the generic node inside your concrete struct**, not the other way around. The kernel does this for `list_head`, `rb_node`, `kref`, `kobject`, `work_struct`, `timer_list`, `hlist_node`, `rcu_head`, etc.

```c
struct my_thing {
    int id;
    char name[32];
    struct list_head list;     /* embedded, not a pointer */
    struct rb_node   tree;     /* same thing in another structure */
    struct kref      ref;
};

/* Iterate the list and recover the my_thing from each node */
struct my_thing *t;
list_for_each_entry(t, &all_things, list) {
    printk("id=%d\n", t->id);
}
```

`list_for_each_entry` is `container_of` in a loop. The same `my_thing` can simultaneously live on a list, in a red-black tree, and be reference-counted, **without paying for an indirection or carrying a back-pointer**. This is the kernel's substitute for templates and inheritance.

**Trap:** `container_of(NULL, ...)` does **not** return `NULL` — it returns an invalid pointer equal to `-offsetof(type, member)`. Always check the source pointer before calling `container_of`.

## 4. GFP flag decision tree (post-2023, after `__GFP_ATOMIC` removal)

```
Where am I being called from?
│
├── Process context, can sleep, no locks held that block reclaim?
│       → GFP_KERNEL                  (the default; 99% of the time this is right)
│
├── Process context, but holding a lock that the FS layer might also take?
│       → GFP_NOFS                    (or use the scope API: memalloc_nofs_save)
│
├── Process context, but holding a lock the I/O layer might also take?
│       → GFP_NOIO                    (or memalloc_noio_save)
│
├── Softirq, tasklet, timer, or anywhere with preemption disabled?
│       → GFP_NOWAIT                  (must handle failure; will not sleep)
│
├── Hard IRQ handler, NMI, or "if this fails the kernel is dying anyway"?
│       → GFP_ATOMIC                  (allowed to dip into reserves; still can fail)
│
└── User-triggered allocation that should be charged to a cgroup?
        → GFP_KERNEL_ACCOUNT          (or `__GFP_ACCOUNT` ORed in)
```

Hard truths most code gets wrong:

- **`GFP_ATOMIC` is not "the safe choice" — it is the *dangerous* choice.** It depletes memory reserves the kernel keeps for its own forward-progress (writing dirty pages out). Overusing it makes the box OOM under loads that would otherwise survive. Use `GFP_NOWAIT` with a fallback if you can possibly afford to fail.
- **`GFP_KERNEL` from inside `rcu_read_lock()` is a deadlock waiting for memory pressure**, because reclaim may try to take an RCU-related lock. Use `GFP_ATOMIC` or, better, allocate before the read-side section.
- **Allocating >`PAGE_SIZE` with `kmalloc` works but fragments.** For "I want N bytes but N could be large", use `kvmalloc` — it tries `kmalloc` first and falls back to `vmalloc`.
- **Always prefer the `kzalloc` / `kcalloc` zeroing variants**, except in measured hot paths. The cost of a memset is invisible compared to the cost of one missed-init bug.

## 5. kref — the only refcounting pattern you should write

```c
struct thing {
    struct kref ref;
    /* ... */
};

static void thing_release(struct kref *ref)
{
    struct thing *t = container_of(ref, struct thing, ref);
    kfree(t);
}

void thing_get(struct thing *t) { kref_get(&t->ref); }
void thing_put(struct thing *t) { kref_put(&t->ref, thing_release); }
```

Why not roll your own atomic counter? Because `kref_put` does the **decrement-and-test atomically**, runs the release function only when the count hits zero, and is paired with the kernel's memory model so the release can't observe a stale state. Hand-rolled `atomic_dec()` followed by `if (count == 0) kfree()` has a window where two CPUs both see "I dropped it to zero" and double-free.

The corollary rule: **the rule of `_get` / `_put` symmetry**. Every function that returns a `struct thing *` either takes a reference (caller must `_put`) or doesn't (caller must not). Document which, in one comment line above the function. Refcount bugs are 90% of the kernel's hardest UAFs.

## 6. Misc rules that are easy to forget

- **`sizeof(*ptr)` always, never `sizeof(struct foo)`.** When `struct foo` is renamed via refactor, the type-based form silently allocates 0 bytes (or the wrong size for an unrelated type) and the bug appears at runtime. The pointer-based form fails to compile.
- **`ARRAY_SIZE(arr)`** for static arrays — never `sizeof(arr)/sizeof(arr[0])` open-coded. The macro has a `__must_be_array` check that fails compilation if `arr` decayed to a pointer (which would silently give 1 or 2).
- **`min_t(type, a, b)` and `max_t(type, a, b)`** when the operands have different types. Plain `min(a, b)` does strict type checking and will refuse to compile, which is the *correct* behavior — fix the types, don't cast.
- **`BUILD_BUG_ON(condition)`** to assert invariants at compile time. Use it to check struct sizes match hardware layouts, alignment of embedded fields, enum values matching protocol constants. A compile-time bug is free; a runtime check is not.
- **`__must_check` on functions whose return value matters.** The compiler will warn callers who ignore it. Use this on every `_get`, every error-returning helper, every "this can fail" function.
- **`/* fall through */` comment** (or `__attribute__((fallthrough))`) on intentional switch-case fallthrough. GCC warns on unmarked fallthrough; the warning catches real bugs.
