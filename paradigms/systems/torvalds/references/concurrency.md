# Concurrency rules — what the compiler and the CPU will do to you⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌​​​‌‍‌‌‌‌​​‌​‍‌​​‌​‌‌‌‍‌​​‌​‌​‌‍​​​​‌​‌​‍​‌​​​‌‌‌⁠‍⁠

Load this when any data is shared across CPUs, threads, or interrupt context. The rules in this file are not best-practice suggestions; they are conditions for the code being correct at all.

## Calling-context table — what you may do where

| Context | May sleep? | May allocate? | Locks you may take | Notes |
|---|---|---|---|---|
| Process, no locks held | yes | `GFP_KERNEL` | mutex, sleeping rwsem, spinlock | The default. |
| Process, holding spinlock | **no** | `GFP_NOWAIT` / `GFP_ATOMIC` only | spinlock (other), no mutex | `mutex_lock` in this context = deadlock. |
| Softirq / tasklet / timer | **no** | `GFP_NOWAIT` / `GFP_ATOMIC` | spin_lock_bh on shared-with-process, plain spinlock on softirq-only | `copy_from_user` here = oops. |
| Hard IRQ handler | **no** | `GFP_ATOMIC` only, sparingly | `spin_lock_irqsave` on shared-with-process | Keep total time under microseconds. |
| NMI | **no** | **no** | nothing that takes any lock at all | Even `printk` is suspect; use `printk_safe` ring. |
| Inside `rcu_read_lock()` | **no** | `GFP_ATOMIC` only, ideally none | spinlock yes, mutex no | The whole section must be bounded and short. |
| Inside `preempt_disable()` | **no** | `GFP_ATOMIC` | spinlock yes | Symmetric to softirq for sleeping rules. |

The non-obvious cell: **inside an RCU read-side critical section, you may not sleep, may not call `synchronize_rcu()`, and may not wait on anything that might wait on a grace period**. Doing so deadlocks the writer side under PREEMPT_RCU=n kernels (the read section becomes infinite).

## READ_ONCE / WRITE_ONCE — when plain accesses are wrong

Plain `x = shared` and `shared = y` are **broken** on shared data, even on x86, even with a single-byte field, even when "it's just one read". The compiler is allowed to:

- **Tear** the load: split a 64-bit read into two 32-bit reads, observing a value that never existed.
- **Refetch**: turn one read into two when register pressure is high; you compare a value to itself but each comparison sees a different snapshot.
- **Hoist or sink** the access across function calls and barriers, as long as it can prove the *single-threaded* program is unchanged.
- **Fold** repeated reads into one, so a polling loop on a flag becomes infinite.
- **Speculate-store**: if the value being stored is a constant, the compiler may invent intermediate stores that other CPUs observe.

`READ_ONCE(x)` and `WRITE_ONCE(x, v)` are `volatile`-cast macros that block all of the above for that single access. They emit no extra instructions on x86. Use them whenever a value is touched by more than one execution context, *even when you also hold a lock* — the compiler does not know about your lock.

```c
/* Wrong: compiler may unroll into one load and spin forever */
while (shared_flag == 0)
    cpu_relax();

/* Right */
while (READ_ONCE(shared_flag) == 0)
    cpu_relax();
```

## RCU — the four rules nobody learns from the docs

1. **Updaters use `rcu_assign_pointer(p, new)`. Readers use `rcu_dereference(p)`.** Plain assignment / dereference is wrong because the compiler can reorder the field initialization with the pointer publication, exposing a half-initialized object to readers. `rcu_assign_pointer` is a release-store; `rcu_dereference` is a consume-load (with barriers on Alpha).
2. **Readers may run concurrently with the updater that called `synchronize_rcu()`. They are not blocked.** That is the entire point of RCU. The grace period waits only for *pre-existing* readers to leave; new readers race in freely.
3. **Never RCU-protect an *index* into an array** — only pointers to objects. Compilers play many more games with integers than with pointers, and they can break the dependency that makes the consume-load work.
4. **Module unload requires `rcu_barrier()`, not `synchronize_rcu()`.** `synchronize_rcu` waits for current readers; `rcu_barrier` waits for all queued callbacks (`call_rcu`) to actually fire. Without it, a callback fires after your `.text` is gone and the kernel oopses with no useful trace.

The pattern for safe RCU swap:

```c
struct gp *new = kmalloc(sizeof(*new), GFP_KERNEL);
new->a = ...;
new->b = ...;

spin_lock(&gp_lock);
old = rcu_dereference_protected(gp, lockdep_is_held(&gp_lock));
rcu_assign_pointer(gp, new);
spin_unlock(&gp_lock);

synchronize_rcu();   /* or call_rcu(&old->rcu, free_cb); */
kfree(old);
```

The reader, anywhere else:

```c
rcu_read_lock();
p = rcu_dereference(gp);
if (p)
    use(p->a, p->b);  /* p is valid until rcu_read_unlock */
rcu_read_unlock();
```

## Memory barriers — the only ones you should know by name

| Barrier | Use when |
|---|---|
| `smp_mb()` | Full ordering, both loads and stores. The big hammer; correct but expensive. |
| `smp_rmb()` | Order earlier loads against later loads. |
| `smp_wmb()` | Order earlier stores against later stores. |
| `smp_store_release(p, v)` | Store with release semantics — pairs with `smp_load_acquire`. Modern preferred form. |
| `smp_load_acquire(p)` | Load with acquire semantics. |
| `barrier()` | Compiler-only barrier, no CPU barrier. Forces a sequence point for the optimizer. |

The "two messages, one consumer" pattern (the only one most code needs):

```c
/* producer */
data = compute();
smp_store_release(&ready, 1);

/* consumer */
if (smp_load_acquire(&ready))
    use(data);   /* guaranteed to see the compute() result */
```

`smp_store_release` / `smp_load_acquire` replace 90% of historical `smp_wmb` / `smp_rmb` usage and are easier to get right because the dependency is local to one variable.

**Trap:** `smp_load_acquire` does not magically *cause* the value you want to be read. It only orders accesses *if* the value is the one that was released. The acquire load can still observe stale memory. If you need "wait until ready", you need a busy-wait or a sleeping wait, not just an acquire.

## Locking choices — the table that takes years to derive

| Primitive | Sleeps? | Cost | Use when |
|---|---|---|---|
| `mutex` | yes | medium, fair | Long critical section, process context only, no irq use. The default for "I need a lock". |
| `spinlock_t` | no | low (uncontended), terrible (contended) | Short critical section (<microseconds), or any IRQ involvement. |
| `spin_lock_irqsave` | no | low | Critical section also entered from hard IRQ. |
| `spin_lock_bh` | no | low | Critical section also entered from softirq/tasklet. |
| `rwlock_t` | no | medium | **Almost never the right answer** — RCU is faster for read-mostly. Avoid. |
| `rw_semaphore` | yes | medium | Read-mostly with rare, expensive writers and need to sleep. |
| `seqlock_t` | no | very low for readers | Reader-mostly, writers rare, readers tolerate retry. Best for time-of-day-style data. |
| `RCU` | no readers / yes writers | ~zero for readers | Reader-dominant, writers rare, readers must not be slowed down at all. |
| `atomic_t` / `atomic64_t` | no | very low | Counter, flag, single-word state. Not a lock. |

**Two failure modes that are not obvious:**

- **`rwlock_t` is almost always slower than RCU and slower than `spinlock_t`** because the cache line still bounces on every reader and writers can starve. Its presence in code is usually a sign of cargo-cult.
- **`seqlock_t` requires readers to retry the entire critical section** if a writer intervenes, which means the read-side cannot have side effects. Readers that allocate, `printk`, or modify state will explode.

## lockdep — the static-analysis-at-runtime tool

Enable `CONFIG_PROVE_LOCKING=y` in any kernel where you write or test new locking code. Lockdep tracks lock-acquisition order and screams (`WARN`) the first time a possible deadlock is observable, *even if it doesn't happen this run*. It catches:

- ABBA inversion (lock A then B in one path, B then A in another).
- Sleeping in atomic context.
- Calling `mutex_lock` while holding a spinlock.
- IRQ-safe lock taken from IRQ-unsafe path.

A lockdep splat is *always* a real bug. Do not silence it; do not annotate around it without understanding why; the false-positive rate is essentially zero in well-instrumented subsystems.

## The single rule that prevents 80% of concurrency bugs

> If two execution contexts can both touch this byte, mark every access with `READ_ONCE` / `WRITE_ONCE` (or stronger) and write a one-line comment naming the protocol that orders them.

The comment is the important half. "Protected by `dev->lock`" or "RCU-protected, written under `gp_lock`" or "atomic, ordered by `release/acquire` on `ready`" — anything but silence. If you can't write the comment, the code isn't correct yet, no matter what the test results say.
